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
    let token_ok = q
        .token
        .as_deref()
        .map(|t| crate::wubi::validate_token(t).is_ok())
        .unwrap_or(false);
    if !token_ok {
        return (StatusCode::UNAUTHORIZED, "无效或过期的登录令牌").into_response();
    }
    match &state.sqlite {
        Some(pool) => {
            let pool = pool.clone();
            ws.on_upgrade(move |socket| handle_socket(socket, pool))
        }
        None => (
            StatusCode::BAD_REQUEST,
            "当前部署不是SQLite数据库，不支持管理功能",
        )
            .into_response(),
    }
}

async fn handle_socket(mut socket: WebSocket, pool: SqlitePool) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(m) => m,
            Err(_) => break,
        };
        if let Message::Text(text) = msg {
            let reply = match serde_json::from_str::<ClientMsg>(&text) {
                Ok(cmd) => process(&pool, cmd).await,
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

async fn process(pool: &SqlitePool, cmd: ClientMsg) -> Value {
    match cmd {
        ClientMsg::Tables => list_tables(pool).await,
        ClientMsg::Schema { table } => table_schema(pool, &table).await,
        ClientMsg::Query { sql } => run_sql(pool, &sql).await,
        ClientMsg::Rows {
            table,
            limit,
            offset,
        } => browse_table(pool, &table, limit, offset).await,
    }
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
    let sql = format!("SELECT * FROM \"{}\" LIMIT ? OFFSET ?", escaped);
    match sqlx::query(&sql).bind(limit).bind(offset).fetch_all(pool).await {
        Ok(rows) => {
            let (columns, data) = rows_to_json(&rows);
            json!({
                "action": "rows",
                "ok": true,
                "table": table,
                "columns": columns,
                "rows": data,
                "limit": limit,
                "offset": offset,
                "elapsed_ms": started.elapsed().as_millis(),
            })
        }
        Err(e) => json!({"action": "rows", "ok": false, "error": e.to_string()}),
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
