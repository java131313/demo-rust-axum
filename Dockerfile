# 多阶段构建：前端构建 -> Rust编译 -> 最小化运行时
# 单个容器内包含：Rust API服务 + 嵌入式前端 + SQLite数据库文件

# ---------- 阶段1：构建前端 ----------
FROM node:20-alpine AS frontend
WORKDIR /fe
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# ---------- 阶段2：编译Rust应用 ----------
FROM rust:1.94-slim AS builder
WORKDIR /usr/src/demo-rust-axum

# 安装编译依赖（sqlx的sqlite为bundled模式，无需系统libsqlite3）
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制依赖文件并预下载（利用Docker缓存层）
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制完整源代码和前端构建产物（rust-embed编译期嵌入二进制）
COPY src ./src
COPY config.yaml ./
COPY data ./data
COPY liaozhai_articles.json ./
COPY swagger-ui-dist ./swagger-ui-dist
COPY --from=frontend /fe/dist ./frontend/dist
RUN touch src/main.rs && cargo build --release

# ---------- 阶段3：最小化运行时 ----------
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
    curl \
    sqlite3 \
    redis-tools \
    && rm -rf /var/lib/apt/lists/*

# AI 编程助手 CLI 工具：multica（AI agent 管理）与 pi（编程助手）
# 默认使用国内镜像源（npmmirror / jsdelivr），可用 --build-arg 覆盖
ARG NODE_VERSION=22.23.1
ARG NODE_DIST_URL=https://registry.npmmirror.com/-/binary/node
ARG NPM_REGISTRY=https://registry.npmmirror.com
ARG PI_VERSION=0.80.2
ARG MULTICA_INSTALL_URL=https://cdn.jsdelivr.net/gh/multica-ai/multica@main/scripts/install.sh

RUN curl -fsSL "${MULTICA_INSTALL_URL}" | bash

# Node.js（pi运行依赖>=22.19），按构建目标架构从镜像安装指定版本
# TARGETARCH(Go命名amd64/arm64) → Node包命名(x64/arm64) 映射
ARG TARGETARCH
RUN set -eux; \
    NODE_ARCH="$TARGETARCH"; \
    if [ "$TARGETARCH" = "amd64" ]; then NODE_ARCH="x64"; fi; \
    FILE="node-v${NODE_VERSION}-linux-${NODE_ARCH}.tar.gz"; \
    curl -fsSL "${NODE_DIST_URL}/v${NODE_VERSION}/${FILE}" -o /tmp/node.tar.gz; \
    tar -xzf /tmp/node.tar.gz -C /usr/local --strip-components=1; \
    rm /tmp/node.tar.gz; \
    node --version

# pi 编程助手（官方npm包，锁定版本与registry）
RUN npm install -g --ignore-scripts --registry="${NPM_REGISTRY}" "@earendil-works/pi-coding-agent@${PI_VERSION}" && pi --version

# 从builder阶段复制编译好的二进制文件
COPY --from=builder /usr/src/demo-rust-axum/target/release/demo-rust-axum ./

# 复制配置文件和五笔字典数据（SQLite首次启动时导入）
COPY --from=builder /usr/src/demo-rust-axum/config.yaml ./
COPY --from=builder /usr/src/demo-rust-axum/data ./data

# 种子数据目录：存储卷挂载会遮蔽/app/data，容器内从/app/seed读取种子文件
COPY --from=builder /usr/src/demo-rust-axum/data ./seed
COPY --from=builder /usr/src/demo-rust-axum/liaozhai_articles.json ./seed/

# SQLite数据目录（可挂载卷持久化）
ENV RUST_LOG=info
ENV SQLITE_PATH=/app/data/app.db

EXPOSE 3000

# 单个命令启动：API + 前端 + 数据库初始化
CMD ["./demo-rust-axum"]
