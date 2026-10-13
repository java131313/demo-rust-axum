# 部署指南：GHCR 镜像 + Sealos

本文档介绍如何将本项目构建为 GitHub Container Registry (GHCR) 镜像并部署到 Sealos。

## 架构

```
┌─────────────────────────────────────────┐
│  单个容器（all-in-one）                    │
│  ┌──────────────┐   ┌────────────────┐  │
│  │ Rust Axum API │   │ 嵌入式Vue前端   │  │
│  └──────┬───────┘   └───────┬────────┘  │
│  ┌──────┴───────────────────┴────────┐  │
│  │   SQLite（主数据库，文件存储）        │  │
│  └──────────────┬────────────────────┘  │
└─────────────────┼───────────────────────┘
                  ▼
        Redis（云端托管，读缓存）
        连接失败自动降级为纯 SQLite
```

- **SQLite**：主数据库，单文件（`SQLITE_PATH`，默认 `/app/data/app.db`），启动时自动建表+导入五笔字典（13.7万条）
- **Redis**：读缓存（`REDIS_URL` 覆盖，config.yaml 中有默认值），缓存热点查询，写入自动失效，**可选**——连不上不影响服务
- **前端**：`frontend/dist` 通过 rust-embed 编译进二进制，SPA 路由自动回退 index.html

## 第一步：构建并推送 GHCR 镜像

### 方式 A：GitHub Actions 自动构建（推荐）

1. 将代码推送到 GitHub 仓库（main 分支）
2. `.github/workflows/docker-publish.yml` 自动构建并推送镜像到：
   `ghcr.io/<你的GitHub用户名>/demo-rust-axum:latest`
3. 首次推送后，需要在 GitHub 仓库 **Settings → Actions → General → Workflow permissions** 确认 `Read and write permissions` 已开启（默认 GITHUB_TOKEN 需要 packages:write 权限）

### 方式 B：本地构建推送

```bash
# 登录 GHCR（需要 GitHub Personal Access Token，含 write:packages 权限）
echo <TOKEN> | docker login ghcr.io -u <你的GitHub用户名> --password-stdin

# 构建并推送
docker buildx build --platform linux/amd64,linux/arm64 \
  -t ghcr.io/<你的GitHub用户名>/demo-rust-axum:latest \
  --push .
```

> 注意：GitHub Packages 的镜像默认**私有**，Sealos 拉取需要配置 imagePullSecret；
> 或在 GHCR 包设置（Packages → 镜像 → Package settings → Change visibility）中设为 Public。

## 第二步：部署到 Sealos

### 方式 A：App Launchpad 图形界面（推荐）

1. Sealos 桌面 → **应用管理 (App Launchpad)** → **新建应用**
2. 填写：
   - 镜像名：`ghcr.io/<你的GitHub用户名>/demo-rust-axum:latest`
   - 端口：3000
   - CPU / 内存：0.5C / 512Mi 起步即可
   - 环境变量（在界面添加）：

     | 名称 | 值 |
     |---|---|
     | `REDIS_URL` | 你的Redis连接串（不填则无缓存，服务照常可用） |
     | `JWT_SECRET` | 一段随机长字符串（必改） |
     | `SQLITE_PATH` | `/app/data/app.db` |

3. 存储：挂载一个容量 ≥1Gi 的卷到 `/app/data`（持久化 SQLite，否则容器重启数据丢失）
4. 创建后，Sealos 会自动分配公网域名；也可在应用详情里绑定自定义域名

### 方式 B：终端 kubectl apply

1. 打开 Sealos 桌面的 **终端 (Terminal)**
2. 修改 `deploy/sealos.yaml` 中的 `ghcr.io/YOUR_GITHUB_USERNAME/demo-rust-axum:latest` 为你的镜像地址
3. 执行：

```bash
kubectl apply -f sealos.yaml
# 在 Sealos 中为 Service "wubi-typing" 开启公网访问（应用管理 → 网络 → 开启外网访问）
```

> `deploy/sealos.yaml` 中 replicas 必须为 1（SQLite 是单文件数据库）。
> 如需多副本，请把数据库换成外部 PostgreSQL/MySQL（代码里保留了对它们的支持）。

## 环境变量一览

| 变量 | 默认值 | 说明 |
|---|---|---|
| `SQLITE_PATH` | `data/app.db`（容器内 `/app/data/app.db`） | SQLite 数据库文件路径 |
| `REDIS_URL` | 空（config.yaml 中为本地占位值） | Redis 缓存连接串，不填则禁用缓存直连 SQLite |
| `JWT_SECRET` | `your-secret-key-change-in-production` | JWT 签名密钥，**生产必改** |
| `RUST_LOG` | `info` | 日志级别 |

> 监听地址（默认 `0.0.0.0:3000`）由 `config.yaml` 的 `server.host` / `server.port` 控制。
> 敏感连接串（Redis密码等）不要写进 git 仓库：本地开发放 `.env` 文件（docker-compose
> 会自动读取；`cargo run` 则用 `REDIS_URL=... cargo run`），Sealos 部署在界面环境变量里填。

## 验证

```bash
curl https://<你的Sealos域名>/api/health
# {"status":"ok","database":"up"}

curl https://<你的Sealos域名>/
# 返回 Vue 前端页面
```
