//! SQLite 数据库管理（类Navicat）：WebSocket 接口
//!
//! 前端通过 /ws/dbadmin?token=<JWT> 连接，发送JSON命令：
//! - {"action":"tables"}                          列出所有表
//! - {"action":"schema","table":"表名"}             查看建表语句
//! - {"action":"rows","table":"表名","limit":100,"offset":0}  分页浏览表数据
//! - {"action":"query","sql":"SELECT ..."}         执行任意SQL
//!
//! 需要有效JWT（与登录同一套令牌），仅SQLite部署支持。

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Column, Pool, Row, Sqlite, SqlitePool};

use crate::wubi::AppState;

#[derive(Deserialize)]
pub struct WsQuery {
    token: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
enum ClientMsg {
    Tables,
    Schema { table: String },
    Query { sql: String },
    Rows {
        table: String,
        #[serde(default = "default_limit")]
        limit: i64,
        #[serde(default)]
        offset: i64,
    },
    /// 更新一行: 按rowid定位, sets为列名->新值(null表示SQL NULL)
    Update {
        table: String,
        rowid: i64,
        sets: std::collections::HashMap<String, Value>,
    },
    /// 新增一行: values为列名->值
    Insert {
        table: String,
        values: std::collections::HashMap<String, Value>,
    },
    /// 删除一行: 按rowid定位
    Delete { table: String, rowid: i64 },
}

fn default_limit() -> i64 {
    100
}

/// WebSocket升级入口：校验JWT + 确认SQLite可用
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(q): Query<WsQuery>,
) -> impl IntoResponse {
    let claims = q
        .token
        .as_deref()
        .and_then(|t| crate::wubi::validate_token(t).ok());
    let claims = match claims {
        Some(c) => c,
        None => return (StatusCode::UNAUTHORIZED, "无效或过期的登录令牌").into_response(),
    };
    match &state.sqlite {
        Some(pool) => {
            let pool = pool.clone();
            ws.on_upgrade(move |socket| handle_socket(socket, pool, claims.sub))
        }
        None => (
            StatusCode::BAD_REQUEST,
            "当前部署不是SQLite数据库，不支持管理功能",
        )
            .into_response(),
    }
}

async fn handle_socket(mut socket: WebSocket, pool: SqlitePool, user_id: String) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(m) => m,
            Err(_) => break,
        };
        if let Message::Text(text) = msg {
            let reply = match serde_json::from_str::<ClientMsg>(&text) {
                Ok(cmd) => process(&pool, cmd, &user_id).await,
                Err(e) => json!({"ok": false, "error": format!("消息格式错误: {}", e)}),
            };
            if socket
                .send(Message::Text(reply.to_string().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    }
}

async fn process(pool: &SqlitePool, cmd: ClientMsg, user_id: &str) -> Value {
    match cmd {
        ClientMsg::Tables => list_tables(pool).await,
        ClientMsg::Schema { table } => table_schema(pool, &table).await,
        ClientMsg::Query { sql } => run_sql(pool, &sql).await,
        ClientMsg::Rows {
            table,
            limit,
            offset,
        } => browse_table(pool, &table, limit, offset).await,
        ClientMsg::Update { table, rowid, sets } => update_row(pool, &table, rowid, sets, user_id).await,
        ClientMsg::Insert { table, values } => insert_row(pool, &table, values, user_id).await,
        ClientMsg::Delete { table, rowid } => delete_row(pool, &table, rowid, user_id).await,
    }
}

/// 将一次修改操作写入审计日志（尽力而为，失败仅告警）
async fn audit(
    pool: &SqlitePool,
    user_id: &str,
    action: &str,
    table: &str,
    rowid: Option<i64>,
    detail: Value,
) {
    // 尽力解析用户名，失败则记录user_id
    let username = match user_id.parse::<i32>() {
        Ok(id) => {
            match sqlx::query_as::<_, (String,)>("SELECT username FROM users WHERE id = ?")
                .bind(id)
                .fetch_optional(pool)
                .await
            {
                Ok(Some((u,))) => u,
                _ => user_id.to_string(),
            }
        }
        Err(_) => user_id.to_string(),
    };

    let _ = sqlx::query(
        "INSERT INTO admin_audit_log (username, action, table_name, rowid, detail) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&username)
    .bind(action)
    .bind(table)
    .bind(rowid)
    .bind(detail.to_string())
    .execute(pool)
    .await;
}

async fn list_tables(pool: &SqlitePool) -> Value {
    let started = std::time::Instant::now();
    let result: Result<Vec<(String,)>, _> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
    )
    .fetch_all(pool)
    .await;
    match result {
        Ok(rows) => json!({
            "action": "tables",
            "ok": true,
            "tables": rows.into_iter().map(|r| r.0).collect::<Vec<_>>(),
            "elapsed_ms": started.elapsed().as_millis(),
        }),
        Err(e) => json!({"action": "tables", "ok": false, "error": e.to_string()}),
    }
}

async fn table_schema(pool: &SqlitePool, table: &str) -> Value {
    let started = std::time::Instant::now();
    match sqlx::query_as::<_, (Option<String>,)>("SELECT sql FROM sqlite_master WHERE name = ?")
        .bind(table)
        .fetch_one(pool)
        .await
    {
        Ok((sql_opt,)) => json!({
            "action": "schema",
            "ok": true,
            "table": table,
            "sql": sql_opt.unwrap_or_default(),
            "elapsed_ms": started.elapsed().as_millis(),
        }),
        Err(e) => json!({"action": "schema", "ok": false, "error": e.to_string()}),
    }
}

async fn browse_table(pool: &SqlitePool, table: &str, limit: i64, offset: i64) -> Value {
    let started = std::time::Instant::now();
    let escaped = table.replace('"', "\"\"");
    // 附带隐藏的rowid用于行编辑/删除定位（每个SQLite行都有，除非WITHOUT ROWID表）
    let sql = format!("SELECT rowid AS __rowid__, * FROM \"{}\" LIMIT ? OFFSET ?", escaped);
    match sqlx::query(&sql).bind(limit).bind(offset).fetch_all(pool).await {
        Ok(rows) => {
            let (raw_columns, data) = rows_to_json(&rows);
            // 第一列是__rowid__，从展示列中剥离，单独作为rowids数组返回
            let rowids: Vec<Value> = data
                .iter()
                .map(|r| r.first().cloned().unwrap_or(Value::Null))
                .collect();
            let columns: Vec<String> = raw_columns.into_iter().skip(1).collect();
            let rows: Vec<Vec<Value>> = data.into_iter().map(|r| r.into_iter().skip(1).collect()).collect();
            json!({
                "action": "rows",
                "ok": true,
                "table": table,
                "columns": columns,
                "rows": rows,
                "rowids": rowids,
                "limit": limit,
                "offset": offset,
                "elapsed_ms": started.elapsed().as_millis(),
            })
        }
        Err(e) => json!({"action": "rows", "ok": false, "error": e.to_string()}),
    }
}

/// 校验列名合法性（字母数字下划线），防止注入
fn valid_ident(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn bind_json<'q>(
    q: sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>>,
    v: &'q Value,
) -> sqlx::query::Query<'q, Sqlite, sqlx::sqlite::SqliteArguments<'q>> {
    match v {
        Value::Null => q.bind(Option::<String>::None),
        Value::Bool(b) => q.bind(*b as i64),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                q.bind(i)
            } else {
                q.bind(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => q.bind(s.clone()),
        // 数组/对象不作为单元格值写入，退化为JSON文本
        other => q.bind(other.to_string()),
    }
}

async fn update_row(
    pool: &SqlitePool,
    table: &str,
    rowid: i64,
    sets: std::collections::HashMap<String, Value>,
    user_id: &str,
) -> Value {
    let started = std::time::Instant::now();
    if sets.is_empty() {
        // 无修改的保存视为安全空操作（前端每行都有保存按钮）
        return json!({"action": "update", "ok": true, "rows_affected": 0, "elapsed_ms": 0});
    }
    if !valid_ident(table) || sets.keys().any(|k| !valid_ident(k)) {
        return json!({"action": "update", "ok": false, "error": "非法的表名或列名"});
    }
    let assignments: Vec<String> = sets.keys().map(|k| format!("\"{}\" = ?", k)).collect();
    let sql = format!(
        "UPDATE \"{}\" SET {} WHERE rowid = ?",
        table,
        assignments.join(", ")
    );
    let mut q = sqlx::query(&sql);
    for v in sets.values() {
        q = bind_json(q, v);
    }
    q = q.bind(rowid);
    match q.execute(pool).await {
        Ok(res) => {
            audit(
                pool,
                user_id,
                "update",
                table,
                Some(rowid),
                json!({"sets": sets}),
            )
            .await;
            json!({
                "action": "update",
                "ok": true,
                "rows_affected": res.rows_affected(),
                "elapsed_ms": started.elapsed().as_millis(),
            })
        }
        Err(e) => json!({"action": "update", "ok": false, "error": e.to_string()}),
    }
}

async fn insert_row(
    pool: &SqlitePool,
    table: &str,
    values: std::collections::HashMap<String, Value>,
    user_id: &str,
) -> Value {
    let started = std::time::Instant::now();
    if values.is_empty() {
        return json!({"action": "insert", "ok": false, "error": "没有要插入的列"});
    }
    if !valid_ident(table) || values.keys().any(|k| !valid_ident(k)) {
        return json!({"action": "insert", "ok": false, "error": "非法的表名或列名"});
    }
    let cols: Vec<String> = values.keys().map(|k| format!("\"{}\"", k)).collect();
    let marks: Vec<&str> = values.keys().map(|_| "?").collect();
    let sql = format!(
        "INSERT INTO \"{}\" ({}) VALUES ({})",
        table,
        cols.join(", "),
        marks.join(", ")
    );
    let mut q = sqlx::query(&sql);
    for v in values.values() {
        q = bind_json(q, v);
    }
    match q.execute(pool).await {
        Ok(res) => {
            audit(
                pool,
                user_id,
                "insert",
                table,
                None,
                json!({"values": values}),
            )
            .await;
            json!({
                "action": "insert",
                "ok": true,
                "rows_affected": res.rows_affected(),
                "elapsed_ms": started.elapsed().as_millis(),
            })
        }
        Err(e) => json!({"action": "insert", "ok": false, "error": e.to_string()}),
    }
}

async fn delete_row(pool: &SqlitePool, table: &str, rowid: i64, user_id: &str) -> Value {
    let started = std::time::Instant::now();
    if !valid_ident(table) {
        return json!({"action": "delete", "ok": false, "error": "非法的表名"});
    }
    let sql = format!("DELETE FROM \"{}\" WHERE rowid = ?", table);
    match sqlx::query(&sql).bind(rowid).execute(pool).await {
        Ok(res) => {
            audit(pool, user_id, "delete", table, Some(rowid), json!({})).await;
            json!({
                "action": "delete",
                "ok": true,
                "rows_affected": res.rows_affected(),
                "elapsed_ms": started.elapsed().as_millis(),
            })
        }
        Err(e) => json!({"action": "delete", "ok": false, "error": e.to_string()}),
    }
}

async fn run_sql(pool: &SqlitePool, sql: &str) -> Value {
    let started = std::time::Instant::now();
    // 先试查询路径（有结果集的语句）
    match sqlx::query(sql).fetch_all(pool).await {
        Ok(rows) => {
            let (columns, data) = rows_to_json(&rows);
            json!({
                "action": "query",
                "ok": true,
                "columns": columns,
                "rows": data,
                "rows_affected": rows.len() as u64,
                "elapsed_ms": started.elapsed().as_millis(),
            })
        }
        Err(fetch_err) => {
            // 回退到执行路径（INSERT/UPDATE/DDL等）
            match sqlx::query(sql).execute(pool).await {
                Ok(res) => json!({
                    "action": "query",
                    "ok": true,
                    "columns": [],
                    "rows": [],
                    "rows_affected": res.rows_affected(),
                    "elapsed_ms": started.elapsed().as_millis(),
                }),
                Err(exec_err) => json!({
                    "action": "query",
                    "ok": false,
                    "error": format!("{}（执行模式：{}）", fetch_err, exec_err),
                }),
            }
        }
    }
}

/// 把sqlx行转换为(列名, JSON值矩阵)；NULL保持为JSON null
fn rows_to_json(rows: &[sqlx::sqlite::SqliteRow]) -> (Vec<String>, Vec<Vec<Value>>) {
    let columns = rows
        .first()
        .map(|r| {
            r.columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let data = rows
        .iter()
        .map(|row| {
            (0..row.columns().len())
                .map(|idx| cell_value(row, idx))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    (columns, data)
}

fn cell_value(row: &sqlx::sqlite::SqliteRow, idx: usize) -> Value {
    if let Ok(v) = row.try_get::<Option<String>, _>(idx) {
        return v.map(Value::from).unwrap_or(Value::Null);
    }
    if let Ok(v) = row.try_get::<Option<i64>, _>(idx) {
        return v.map(Value::from).unwrap_or(Value::Null);
    }
    if let Ok(v) = row.try_get::<Option<f64>, _>(idx) {
        return v.map(Value::from).unwrap_or(Value::Null);
    }
    if let Ok(v) = row.try_get::<Option<Vec<u8>>, _>(idx) {
        return v
            .map(|b| Value::from(format!("BLOB({} bytes)", b.len())))
            .unwrap_or(Value::Null);
    }
    Value::Null
}

/// 便于测试和外部使用
#[allow(dead_code)]
pub async fn ping_pool(pool: &Pool<Sqlite>) -> bool {
    sqlx::query("SELECT 1").fetch_one(pool).await.is_ok()
}
