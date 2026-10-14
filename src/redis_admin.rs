//! Redis 缓存管理：WebSocket 接口（类Redis桌面管理工具）
//!
//! 前端通过 /ws/redisadmin?token=<JWT> 连接，JSON命令：
//! - {"action":"scan","pattern":"*","cursor":0}     分页列出key（SCAN，托管Redis禁用KEYS）
//! - {"action":"get","key":"..."}                   按类型读取值（string/hash/list/set/zset）
//! - {"action":"set","key":"...","value":"..."}     写入string类型
//! - {"action":"hset","key":"...","field":"...","value":"..."}  写入hash字段
//! - {"action":"hdel","key":"...","field":"..."}    删除hash字段
//! - {"action":"add_elem","key":"...","kind":"list|set|zset","value":"..."}  添加元素（zset值格式"score member"）
//! - {"action":"del_elem","key":"...","kind":"list|set|zset","value":"..."}  删除元素
//! - {"action":"del","key":"..."}                   删除整个key
//! - {"action":"clear","pattern":"..."}             按前缀批量清除（SCAN+DEL分批）

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::IntoResponse,
};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::wubi::AppState;

#[derive(Deserialize)]
pub struct WsQuery {
    token: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
enum ClientMsg {
    Scan {
        #[serde(default = "default_pattern")]
        pattern: String,
        #[serde(default)]
        cursor: u64,
    },
    Get { key: String },
    Set { key: String, value: String },
    HSet {
        key: String,
        field: String,
        value: String,
    },
    HDel { key: String, field: String },
    AddElem {
        key: String,
        kind: String,
        value: String,
    },
    DelElem {
        key: String,
        kind: String,
        value: String,
    },
    Del { key: String },
    Clear {
        #[serde(default = "default_pattern")]
        pattern: String,
    },
}

fn default_pattern() -> String {
    "*".to_string()
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(q): Query<WsQuery>,
) -> impl IntoResponse {
    let claims = q
        .token
        .as_deref()
        .and_then(|t| crate::wubi::validate_token(t).ok());
    if claims.is_none() {
        return (StatusCode::UNAUTHORIZED, "无效或过期的登录令牌").into_response();
    }
    match &state.redis {
        Some(conn) => {
            let conn = conn.clone();
            ws.on_upgrade(move |socket| handle_socket(socket, conn))
        }
        None => (StatusCode::BAD_REQUEST, "未启用Redis缓存").into_response(),
    }
}

async fn handle_socket(mut socket: WebSocket, conn: ConnectionManager) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(m) => m,
            Err(_) => break,
        };
        if let Message::Text(text) = msg {
            let reply = match serde_json::from_str::<ClientMsg>(&text) {
                Ok(cmd) => process(&conn, cmd).await,
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

async fn process(conn: &ConnectionManager, cmd: ClientMsg) -> Value {
    match cmd {
        ClientMsg::Scan { pattern, cursor } => scan_keys(conn, &pattern, cursor).await,
        ClientMsg::Get { key } => get_key(conn, &key).await,
        ClientMsg::Set { key, value } => {
            run(conn, "set", &key, async {
                let mut c = conn.clone();
                c.set::<_, _, ()>(&key, &value).await
            })
            .await
        }
        ClientMsg::HSet { key, field, value } => {
            run(conn, "hset", &key, async {
                let mut c = conn.clone();
                c.hset::<_, _, _, ()>(&key, &field, &value).await
            })
            .await
        }
        ClientMsg::HDel { key, field } => {
            run(conn, "hdel", &key, async {
                let mut c = conn.clone();
                c.hdel::<_, _, ()>(&key, &field).await
            })
            .await
        }
        ClientMsg::AddElem { key, kind, value } => add_elem(conn, &key, &kind, &value).await,
        ClientMsg::DelElem { key, kind, value } => del_elem(conn, &key, &kind, &value).await,
        ClientMsg::Del { key } => {
            run(conn, "del", &key, async {
                let mut c = conn.clone();
                c.del::<_, ()>(&key).await
            })
            .await
        }
        ClientMsg::Clear { pattern } => clear_by_pattern(conn, &pattern).await,
    }
}

/// 通用执行封装：成功返回ok，失败返回error
async fn run<F>(
    _conn: &ConnectionManager,
    action: &str,
    key: &str,
    fut: F,
) -> Value
where
    F: std::future::Future<Output = redis::RedisResult<()>>,
{
    match fut.await {
        Ok(()) => json!({"action": action, "ok": true, "key": key}),
        Err(e) => json!({"action": action, "ok": false, "key": key, "error": e.to_string()}),
    }
}

/// SCAN分页列key（托管Redis通常禁用KEYS命令，必须用SCAN）
async fn scan_keys(conn: &ConnectionManager, pattern: &str, cursor: u64) -> Value {
    let mut c = conn.clone();
    let result: redis::RedisResult<(u64, Vec<String>)> = redis::cmd("SCAN")
        .arg(cursor)
        .arg("MATCH")
        .arg(pattern)
        .arg("COUNT")
        .arg(200)
        .query_async(&mut c)
        .await;
    match result {
        Ok((next_cursor, keys)) => {
            // 批量取类型与TTL
            let mut entries = Vec::new();
            for key in keys {
                let mut cc = conn.clone();
                let key_clone = key.clone();
                let info: redis::RedisResult<(String, i64)> = async {
                    let t: String = redis::cmd("TYPE").arg(&key_clone).query_async(&mut cc).await?;
                    let ttl: i64 = cc.ttl(&key_clone).await?;
                    Ok((t, ttl))
                }
                .await;
                let (typ, ttl) = info.unwrap_or_else(|_| ("unknown".into(), -2));
                entries.push(json!({"key": key, "type": typ, "ttl": ttl}));
            }
            json!({
                "action": "scan",
                "ok": true,
                "keys": entries,
                "next_cursor": next_cursor,
            })
        }
        Err(e) => json!({"action": "scan", "ok": false, "error": e.to_string()}),
    }
}

/// 按类型读取key值
async fn get_key(conn: &ConnectionManager, key: &str) -> Value {
    let mut c = conn.clone();
    let typ: String = match redis::cmd("TYPE").arg(key).query_async(&mut c).await {
        Ok(t) => t,
        Err(e) => return json!({"action": "get", "ok": false, "error": e.to_string()}),
    };
    let mut cc = conn.clone();
    let value = match typ.as_str() {
        "string" => {
            let v: Option<String> = cc.get(key).await.unwrap_or(None);
            json!({"type": "string", "value": v})
        }
        "hash" => {
            let pairs: Vec<(String, String)> = cc.hgetall(key).await.unwrap_or_default();
            json!({"type": "hash", "entries": pairs})
        }
        "list" => {
            let items: Vec<String> = cc.lrange(key, 0, -1).await.unwrap_or_default();
            json!({"type": "list", "entries": items})
        }
        "set" => {
            let items: Vec<String> = cc.smembers(key).await.unwrap_or_default();
            json!({"type": "set", "entries": items})
        }
        "zset" => {
            let items: Vec<(String, f64)> = cc
                .zrange_withscores(key, 0, -1)
                .await
                .unwrap_or_default();
            json!({"type": "zset", "entries": items})
        }
        other => json!({"type": other, "value": null}),
    };
    json!({"action": "get", "ok": true, "key": key, "data": value})
}

async fn add_elem(conn: &ConnectionManager, key: &str, kind: &str, value: &str) -> Value {
    let mut c = conn.clone();
    let result = match kind {
        "list" => c.rpush::<_, _, ()>(key, value).await,
        "set" => c.sadd::<_, _, ()>(key, value).await,
        "zset" => {
            // 值格式: "score member"
            let (score, member) = match value.split_once(' ') {
                Some((s, m)) => (s.parse::<f64>().unwrap_or(0.0), m.to_string()),
                None => (0.0, value.to_string()),
            };
            c.zadd::<_, _, _, ()>(key, member, score).await
        }
        _ => return json!({"action": "add_elem", "ok": false, "error": "不支持的类型"}),
    };
    match result {
        Ok(()) => json!({"action": "add_elem", "ok": true, "key": key}),
        Err(e) => json!({"action": "add_elem", "ok": false, "error": e.to_string()}),
    }
}

async fn del_elem(conn: &ConnectionManager, key: &str, kind: &str, value: &str) -> Value {
    let mut c = conn.clone();
    let result = match kind {
        "list" => c.lrem::<_, _, ()>(key, 0, value).await,
        "set" => c.srem::<_, _, ()>(key, value).await,
        "zset" => c.zrem::<_, _, ()>(key, value).await,
        _ => return json!({"action": "del_elem", "ok": false, "error": "不支持的类型"}),
    };
    match result {
        Ok(()) => json!({"action": "del_elem", "ok": true, "key": key}),
        Err(e) => json!({"action": "del_elem", "ok": false, "error": e.to_string()}),
    }
}

/// 按前缀批量清除：SCAN+DEL分批，单次最多处理100批（约1万key）防止WS长时间阻塞
async fn clear_by_pattern(conn: &ConnectionManager, pattern: &str) -> Value {
    let mut cursor: u64 = 0;
    let mut deleted: u64 = 0;
    for _ in 0..100 {
        let mut c = conn.clone();
        let result: redis::RedisResult<(u64, Vec<String>)> = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(pattern)
            .arg("COUNT")
            .arg(100)
            .query_async(&mut c)
            .await;
        match result {
            Ok((next, keys)) => {
                if !keys.is_empty() {
                    let mut cc = conn.clone();
                    match cc.del::<_, u64>(&keys).await {
                        Ok(n) => deleted += n,
                        Err(e) => {
                            return json!({"action": "clear", "ok": false,
                                "deleted": deleted, "error": e.to_string()})
                        }
                    }
                }
                cursor = next;
                if cursor == 0 {
                    return json!({"action": "clear", "ok": true, "deleted": deleted, "finished": true});
                }
            }
            Err(e) => {
                return json!({"action": "clear", "ok": false,
                    "deleted": deleted, "error": e.to_string()})
            }
        }
    }
    json!({"action": "clear", "ok": true, "deleted": deleted, "finished": false})
}
