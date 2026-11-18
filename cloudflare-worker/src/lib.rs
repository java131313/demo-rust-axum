//! 五笔打字练习 — Cloudflare Worker 移植版（Rust → WASM）
//!
//! 架构：workers-rs Router + Cloudflare D1（SQLite兼容）+ 前端dist嵌入WASM服务
//! 与 axum 版 API 契约一致；数据库管理/Redis管理两个WS tab 在Worker上不可用（优雅降级）。

use base64::Engine;
use worker::d1::D1Type;
use hmac::{Hmac, Mac};
use include_dir::{include_dir, Dir};
use serde_json::{json, Value};
use sha2::Sha256;
use worker::*;

type HmacSha256 = Hmac<Sha256>;

static DIST: Dir = include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist");

const TOKEN_EXPIRATION_SECS: i64 = 86400; // 24小时，与axum版一致

// ============ JWT (HS256, 纯Rust wasm兼容) ============

fn jwt_secret(env: &Env) -> String {
    env.var("JWT_SECRET")
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "your-secret-key".to_string())
}

fn b64u(data: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

fn b64u_decode(s: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .ok()
}

fn sign_jwt(user_id: i32, env: &Env) -> String {
    let header = b64u(br#"{"alg":"HS256","typ":"JWT"}"#);
    let exp = (Date::now().as_millis() as i64 / 1000) + TOKEN_EXPIRATION_SECS;
    let payload = b64u(&json!({"sub": user_id.to_string(), "exp": exp}).to_string().into_bytes());
    let msg = format!("{}.{}", header, payload);
    let mut mac = HmacSha256::new_from_slice(jwt_secret(env).as_bytes())
        .expect("HMAC init failed");
    mac.update(msg.as_bytes());
    let sig = b64u(&mac.finalize().into_bytes());
    format!("{}.{}", msg, sig)
}

fn verify_jwt(token: &str, env: &Env) -> Option<i32> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let msg = format!("{}.{}", parts[0], parts[1]);
    let mut mac = HmacSha256::new_from_slice(jwt_secret(env).as_bytes()).ok()?;
    mac.update(msg.as_bytes());
    let expect = mac.finalize().into_bytes();
    let actual = b64u_decode(parts[2])?;
    if expect.as_slice() != actual.as_slice() {
        return None;
    }
    let payload: Value = serde_json::from_slice(&b64u_decode(parts[1])?).ok()?;
    let exp = payload.get("exp")?.as_i64()?;
    let now = Date::now().as_millis() as i64 / 1000;
    if now > exp {
        return None;
    }
    payload.get("sub")?.as_str()?.parse().ok()
}

fn auth_user_id(req: &Request, env: &Env) -> Option<i32> {
    let header = req.headers().get("Authorization").ok()??;
    let token = header.strip_prefix("Bearer ")?;
    verify_jwt(token, env)
}

fn err(status: u16, msg: &str) -> Result<Response> {
    Ok(Response::from_json(&json!({"error": msg}))?.with_status(status))
}

// ============ D1 辅助 ============

fn db(ctx: &RouteContext<()>) -> Result<D1Database> {
    ctx.env.d1("DB")
}

/// serde_json::Value -> D1Type（JSON数组/对象退化为JSON文本）
fn to_d1(v: &Value) -> D1Type<'_> {
    match v {
        Value::Null => D1Type::Null,
        Value::Bool(b) => D1Type::Boolean(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                D1Type::Integer(i as i32)
            } else {
                D1Type::Real(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => D1Type::Text(s),
        other => D1Type::Text(Box::leak(other.to_string().into_boxed_str())),
    }
}

fn bind_stmt<'a>(db: &'a D1Database, sql: &str, binds: &'a [Value]) -> Result<D1PreparedStatement> {
    let stmt = db.prepare(sql);
    if binds.is_empty() {
        Ok(stmt)
    } else {
        let args: Vec<D1Type> = binds.iter().map(to_d1).collect();
        stmt.bind_refs(args.iter())
    }
}

async fn query_all(db: &D1Database, sql: &str, binds: &[Value]) -> Result<Vec<Value>> {
    let stmt = bind_stmt(db, sql, binds)?;
    let res = stmt.all().await?;
    Ok(res.results::<Value>().unwrap_or_default())
}

async fn query_one(db: &D1Database, sql: &str, binds: &[Value]) -> Result<Option<Value>> {
    let stmt = bind_stmt(db, sql, binds)?;
    stmt.first::<Value>(None).await
}

async fn exec(db: &D1Database, sql: &str, binds: &[Value]) -> Result<()> {
    let stmt = bind_stmt(db, sql, binds)?;
    stmt.run().await?;
    Ok(())
}

async fn last_insert_id(db: &D1Database) -> Result<i64> {
    let row = query_one(db, "SELECT last_insert_rowid() AS id", &[]).await?;
    Ok(row.and_then(|r| r.get("id").and_then(|v| v.as_i64())).unwrap_or(0))
}

// ============ 静态资源（前端嵌入WASM，SPA回退index.html） ============

fn mime_of(path: &str) -> &'static str {
    if path.ends_with(".js") {
        "text/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".txt") {
        "text/plain; charset=utf-8"
    } else {
        "application/octet-stream"
    }
}

fn serve_static(path: &str) -> Result<Response> {
    let path = path.trim_start_matches('/');
    // 先尝试精确文件（带扩展名），否则SPA回退index.html（vue-router路径）
    let resolved = if !path.is_empty() && path.contains('.') && DIST.get_file(path).is_some() {
        path.to_string()
    } else {
        "index.html".to_string()
    };
    match DIST.get_file(&resolved) {
        Some(f) => {
            let mut resp = Response::from_bytes(f.contents().to_vec())?;
            resp.headers_mut().set("Content-Type", mime_of(&resolved))?;
            Ok(resp)
        }
        None => Response::error("Not Found", 404),
    }
}

// ============ 处理器 ============

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let url = req.url()?;
    let path = url.path().to_string();
    let path_str = path.as_str();

    // WS管理端点在Worker不可用（无SQLite/Redis直连），返回明确提示
    if path_str.starts_with("/ws/") {
        return Ok(Response::from_json(&json!({"ok": false, "error": "Worker部署不支持数据库/Redis管理（见Sealos/Render部署）"}))?.with_status(501));
    }

    if !path_str.starts_with("/api/") {
        // GET/HEAD 之外的静态请求直接回退
        return serve_static(path_str);
    }

    let router = Router::new()
        // ===== 健康 =====
        .get_async("/api/health", |_, _| async {
            Response::from_json(&json!({"status": "ok", "database": "up"}))
        })
        // ===== 认证 =====
        .post_async("/api/register", |mut req, ctx| async move {
            let body: Value = req.json().await?;
            let username = body["username"].as_str().unwrap_or("").to_string();
            let email = body["email"].as_str().unwrap_or("").to_string();
            let password = body["password"].as_str().unwrap_or("").to_string();
            if username.is_empty() || email.is_empty() || password.is_empty() {
                return err(400, "用户名/邮箱/密码不能为空");
            }
            let db = db(&ctx)?;
            let exists = query_one(&db, "SELECT id FROM users WHERE username = ?1 OR email = ?2",
                &[json!(username), json!(email)]).await?;
            if exists.is_some() {
                return err(409, "用户名或邮箱已存在");
            }
            // cost 4：Worker免费档CPU时间受限，演示项目可接受
            let hash = bcrypt::hash(&password, 4).map_err(|e| Error::RustError(e.to_string()))?;
            exec(&db, "INSERT INTO users (username, email, password_hash) VALUES (?1, ?2, ?3)",
                &[json!(username), json!(email), json!(hash)]).await?;
            let id = last_insert_id(&db).await?;
            let token = sign_jwt(id as i32, &ctx.env);
            Response::from_json(&json!({
                "access_token": token,
                "user": {"id": id, "username": username, "email": email}
            }))
        })
        .post_async("/api/login", |mut req, ctx| async move {
            let body: Value = req.json().await?;
            let username = body["username"].as_str().unwrap_or("").to_string();
            let password = body["password"].as_str().unwrap_or("").to_string();
            let db = db(&ctx)?;
            let user = query_one(&db, "SELECT * FROM users WHERE username = ?1", &[json!(username)]).await?;
            let user = match user {
                Some(u) => u,
                None => return err(401, "用户名或密码错误"),
            };
            let hash = user["password_hash"].as_str().unwrap_or("");
            if !bcrypt::verify(&password, hash).unwrap_or(false) {
                return err(401, "用户名或密码错误");
            }
            let token = sign_jwt(user["id"].as_i64().unwrap_or(0) as i32, &ctx.env);
            Response::from_json(&json!({
                "access_token": token,
                "user": {
                    "id": user["id"],
                    "username": user["username"],
                    "email": user["email"],
                    "created_at": user["created_at"]
                }
            }))
        })
        .post_async("/api/logout", |_, _| async {
            Response::from_json(&json!({"message": "已退出"}))
        })
        .get_async("/api/me", |req, ctx| async move {
            let uid = match auth_user_id(&req, &ctx.env) {
                Some(id) => id,
                None => return err(401, "未登录"),
            };
            let db = db(&ctx)?;
            let user = query_one(&db, "SELECT id, username, email, created_at FROM users WHERE id = ?1",
                &[json!(uid)]).await?;
            match user {
                Some(u) => Response::from_json(&u),
                None => err(404, "用户不存在"),
            }
        })
        // ===== 课程与进度 =====
        .get_async("/api/lessons", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM lessons", &[]).await?)
        })
        .get_async("/api/lessons/:id", |_, ctx| async move {
            let db = db(&ctx)?;
            let id = ctx.param("id").map(|s| s.as_str()).unwrap_or("0").to_string();
            match query_one(&db, "SELECT * FROM lessons WHERE id = ?1", &[json!(id)]).await? {
                Some(row) => Response::from_json(&row),
                None => err(404, "课程不存在"),
            }
        })
        .post_async("/api/lessons", |mut req, ctx| async move {
            let body: Value = req.json().await?;
            let db = db(&ctx)?;
            exec(&db, "INSERT INTO lessons (character_val, code, description) VALUES (?1, ?2, ?3)",
                &[body["character"].clone(), body["code"].clone(), body["description"].clone()]).await?;
            let id = last_insert_id(&db).await?;
            Response::from_json(&json!({"id": id}))
        })
        .post_async("/api/progress", |mut req, ctx| async move {
            let body: Value = req.json().await?;
            let db = db(&ctx)?;
            exec(&db, "INSERT INTO user_progress (user_name, lesson_id, accuracy, score) VALUES (?1, ?2, ?3, ?4)",
                &[body["user_name"].clone(), body["lesson_id"].clone(),
                  body["accuracy"].clone(), body["score"].clone()]).await?;
            Ok(Response::from_json(&json!({"status": "ok"}))?.with_status(201))
        })
        // ===== 系统文章 =====
        .get_async("/api/articles", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM articles", &[]).await?)
        })
        .post_async("/api/articles", |mut req, ctx| async move {
            let body: Value = req.json().await?;
            let db = db(&ctx)?;
            exec(&db, "INSERT INTO articles (title, content, difficulty) VALUES (?1, ?2, ?3)",
                &[body["title"].clone(), body["content"].clone(),
                  body["difficulty"].clone()]).await?;
            let id = last_insert_id(&db).await?;
            Response::from_json(&json!({"id": id}))
        })
        .get_async("/api/articles/:id", |_, ctx| async move {
            let db = db(&ctx)?;
            let id = ctx.param("id").map(|s| s.as_str()).unwrap_or("0").to_string();
            match query_one(&db, "SELECT * FROM articles WHERE id = ?1", &[json!(id)]).await? {
                Some(row) => Response::from_json(&row),
                None => err(404, "文章不存在"),
            }
        })
        .put_async("/api/articles/:id", |mut req, ctx| async move {
            let body: Value = req.json().await?;
            let db = db(&ctx)?;
            let id = ctx.param("id").map(|s| s.as_str()).unwrap_or("0").to_string();
            exec(&db, "UPDATE articles SET title = ?1, content = ?2, difficulty = ?3 WHERE id = ?4",
                &[body["title"].clone(), body["content"].clone(),
                  body["difficulty"].clone(), json!(id)]).await?;
            Response::from_json(&json!({"id": id}))
        })
        .delete_async("/api/articles/:id", |_, ctx| async move {
            let db = db(&ctx)?;
            let id = ctx.param("id").map(|s| s.as_str()).unwrap_or("0").to_string();
            exec(&db, "DELETE FROM articles WHERE id = ?1", &[json!(id)]).await?;
            Ok(Response::empty()?.with_status(204))
        })
        // ===== 自定义文章（聊斋等）=====
        .get_async("/api/custom-articles", |req, ctx| async move {
            let url = req.url()?;
            let lang = url.query_pairs()
                .find(|(k, _)| k == "lang")
                .map(|(_, v)| v.to_string())
                .unwrap_or_else(|| "zh".to_string());
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM custom_articles WHERE lang = ?1 ORDER BY id",
                &[json!(lang)]).await?)
        })
        .post_async("/api/custom-articles", |mut req, ctx| async move {
            if auth_user_id(&req, &ctx.env).is_none() {
                return err(401, "请先登录");
            }
            let body: Value = req.json().await?;
            let db = db(&ctx)?;
            exec(&db, "INSERT INTO custom_articles (title, content, difficulty, lang) VALUES (?1, ?2, ?3, ?4)",
                &[body["title"].clone(), body["content"].clone(),
                  body["difficulty"].clone(),
                  body["lang"].clone()]).await?;
            let id = last_insert_id(&db).await?;
            Response::from_json(&json!({"id": id, "title": body["title"], "content": body["content"], "difficulty": body["difficulty"]}))
        })
        .put_async("/api/custom-articles/:id", |mut req, ctx| async move {
            if auth_user_id(&req, &ctx.env).is_none() {
                return err(401, "请先登录");
            }
            let body: Value = req.json().await?;
            let db = db(&ctx)?;
            let id = ctx.param("id").map(|s| s.as_str()).unwrap_or("0").to_string();
            // 合并已有记录，未提供的字段保持不变
            let existing = query_one(&db, "SELECT * FROM custom_articles WHERE id = ?1", &[json!(id)]).await?;
            let existing = match existing {
                Some(e) => e,
                None => return err(404, "文章不存在"),
            };
            let title = body.get("title").cloned().unwrap_or(existing["title"].clone());
            let content = body.get("content").cloned().unwrap_or(existing["content"].clone());
            let difficulty = body.get("difficulty").cloned().unwrap_or(existing["difficulty"].clone());
            exec(&db, "UPDATE custom_articles SET title = ?1, content = ?2, difficulty = ?3 WHERE id = ?4",
                &[title.clone(), content.clone(), difficulty.clone(), json!(id)]).await?;
            Response::from_json(&json!({"id": id, "title": title, "content": content, "difficulty": difficulty}))
        })
        .delete_async("/api/custom-articles/:id", |req, ctx| async move {
            if auth_user_id(&req, &ctx.env).is_none() {
                return err(401, "请先登录");
            }
            let db = db(&ctx)?;
            let id = ctx.param("id").map(|s| s.as_str()).unwrap_or("0").to_string();
            exec(&db, "DELETE FROM custom_articles WHERE id = ?1", &[json!(id)]).await?;
            Ok(Response::empty()?.with_status(204))
        })
        // ===== 五笔 =====
        .get_async("/api/wubi/:character", |_, ctx| async move {
            let db = db(&ctx)?;
            let ch = ctx.param("character").map(|s| s.as_str()).unwrap_or("").to_string();
            let ch = urlencoding::decode(&ch).map(|c| c.into_owned()).unwrap_or(ch);
            let row = query_one(&db, "SELECT * FROM wubi_characters WHERE character_val = ?1", &[json!(ch)]).await?;
            match row {
                Some(r) => Response::from_json(&r),
                // 与axum版一致：未收录返回id=0占位
                None => Response::from_json(&json!({
                    "id": 0, "character": ch, "simple_code": "", "full_code": "",
                    "pinyin": "", "remark": "", "created_at": null
                })),
            }
        })
        .put_async("/api/wubi-code", |mut req, ctx| async move {
            let body: Value = req.json().await?;
            let db = db(&ctx)?;
            let ch = body["character"].as_str().unwrap_or("").to_string();
            let code = body["code"].as_str().unwrap_or("").to_string();
            if ch.is_empty() {
                return err(400, "character不能为空");
            }
            let existing = query_one(&db, "SELECT * FROM wubi_characters WHERE character_val = ?1", &[json!(ch)]).await?;
            let row = if existing.is_some() {
                exec(&db, "UPDATE wubi_characters SET simple_code = ?1, full_code = ?1 WHERE character_val = ?2",
                    &[json!(code), json!(ch)]).await?;
                query_one(&db, "SELECT * FROM wubi_characters WHERE character_val = ?1", &[json!(ch)]).await?.unwrap_or(json!({}))
            } else {
                exec(&db, "INSERT INTO wubi_characters (character_val, simple_code, full_code) VALUES (?1, ?2, ?3)",
                    &[json!(ch), json!(code), json!(code)]).await?;
                query_one(&db, "SELECT * FROM wubi_characters WHERE character_val = ?1", &[json!(ch)]).await?.unwrap_or(json!({}))
            };
            Response::from_json(&row)
        })
        .get_async("/api/wubi-roots", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM wubi_roots", &[]).await?)
        })
        .get_async("/api/wubi-roots/:id", |_, ctx| async move {
            let db = db(&ctx)?;
            let id = ctx.param("id").map(|s| s.as_str()).unwrap_or("0").to_string();
            match query_one(&db, "SELECT * FROM wubi_roots WHERE id = ?1", &[json!(id)]).await? {
                Some(row) => Response::from_json(&row),
                None => err(404, "不存在"),
            }
        })
        .get_async("/api/search-wubi-root/:character", |_, ctx| async move {
            let db = db(&ctx)?;
            let ch = ctx.param("character").map(|s| s.as_str()).unwrap_or("").to_string();
            let ch = urlencoding::decode(&ch).map(|c| c.into_owned()).unwrap_or(ch);
            let rows = query_all(&db, "SELECT * FROM wubi_roots WHERE character_val = ?1", &[json!(ch)]).await?;
            Response::from_json(&rows)
        })
        .post_async("/api/wubi-roots", |mut req, ctx| async move {
            let body: Value = req.json().await?;
            let db = db(&ctx)?;
            exec(&db, "INSERT INTO wubi_roots (character_val, code, position, description) VALUES (?1, ?2, ?3, ?4)",
                &[body["character"].clone(), body["code"].clone(),
                  body["position"].clone(), body["description"].clone()]).await?;
            let id = last_insert_id(&db).await?;
            Response::from_json(&json!({"id": id}))
        })
        .get_async("/api/key-radicals", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM key_radicals", &[]).await?)
        })
        .get_async("/api/key-radicals/:key", |_, ctx| async move {
            let db = db(&ctx)?;
            let k = ctx.param("key").map(|s| s.as_str()).unwrap_or("").to_string();
            match query_one(&db, "SELECT * FROM key_radicals WHERE key_char = ?1", &[json!(k)]).await? {
                Some(row) => Response::from_json(&row),
                None => err(404, "不存在"),
            }
        })
        // ===== 内容库（静态查询）=====
        .get_async("/api/english-texts", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM english_texts", &[]).await?)
        })
        .get_async("/api/japanese-texts", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM japanese_texts", &[]).await?)
        })
        .get_async("/api/japanese-keyboards", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM japanese_keyboards", &[]).await?)
        })
        .get_async("/api/japanese-characters", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM japanese_characters", &[]).await?)
        })
        .get_async("/api/traditional-chinese-texts", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM traditional_chinese_texts", &[]).await?)
        })
        .get_async("/api/bopomofo-keyboards", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM bopomofo_keyboards", &[]).await?)
        })
        .get_async("/api/bopomofo-characters", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM bopomofo_characters", &[]).await?)
        })
        .get_async("/api/japanese-gojuon", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM japanese_gojuon", &[]).await?)
        })
        .get_async("/api/japanese-kanji", |_, ctx| async move {
            let db = db(&ctx)?;
            Response::from_json(&query_all(&db, "SELECT * FROM japanese_kanji", &[]).await?)
        })
        .get_async("/api/japanese-kanji/:kanji", |_, ctx| async move {
            let db = db(&ctx)?;
            let k = ctx.param("kanji").map(|s| s.as_str()).unwrap_or("").to_string();
            let k = urlencoding::decode(&k).map(|c| c.into_owned()).unwrap_or(k);
            Response::from_json(&query_all(&db, "SELECT * FROM japanese_kanji WHERE kanji = ?1", &[json!(k)]).await?)
        });

    router.run(req, env).await
}
