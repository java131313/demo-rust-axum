# 五笔打字练习 — Cloudflare Worker 移植版

用 **workers-rs（Rust→WASM）** 将 demo-rust-axum 后端移植到 Cloudflare Workers，
数据库使用 **Cloudflare D1**（云版 SQLite），前端 dist 直接嵌入 WASM 由 Worker 自身服务
（与 axum 版同一套 `frontend/dist`，前后端一体）。

## 与 axum 版的差异

| 能力 | axum 版 | Worker 版 |
|---|---|---|
| 数据库 | SQLite 文件 / PostgreSQL | D1（迁移文件在 `migrations/`） |
| 缓存 | Redis（可选） | 无（D1 就近读取足够快） |
| 数据库管理 tab | ✅ SQLite 行编辑 | ❌ 返回 501 提示 |
| Redis 管理 tab | ✅ | ❌ 返回 501 提示 |
| REST API / 前端 | 完全一致 | 完全一致 |

## 本地开发

```bash
cd cloudflare-worker
cargo install worker-build --version 0.1.14   # 与 worker 0.5 匹配，仅需一次
worker-build --release
npx wrangler@3 d1 migrations apply wubi-typing --local
npx wrangler@3 dev --local
# 打开 http://localhost:8787
```

## 部署（首次）

```bash
npx wrangler login                              # 浏览器授权
npx wrangler d1 create wubi-typing              # 创建远程D1，复制输出的 database_id
# 把 wrangler.toml 里的 database_id = "local-only" 替换为真实ID
npx wrangler secret put JWT_SECRET              # 输入随机密钥
npx wrangler d1 migrations apply wubi-typing --remote   # 灌入13.2万字库+494篇聊斋（约数分钟）
npx wrangler deploy                             # 部署
```

之后每次更新：`worker-build --release && npx wrangler deploy`。

## 数据迁移再生成

若 axum 版数据更新（新字库/新文章），重新生成 D1 迁移：

```bash
python3 -I scripts/gen_d1_migrations.py   # 从 data/app.db 导出 schema+数据
```
