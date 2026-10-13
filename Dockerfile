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
    && rm -rf /var/lib/apt/lists/*

# 从builder阶段复制编译好的二进制文件
COPY --from=builder /usr/src/demo-rust-axum/target/release/demo-rust-axum ./

# 复制配置文件和五笔字典数据（SQLite首次启动时导入）
COPY --from=builder /usr/src/demo-rust-axum/config.yaml ./
COPY --from=builder /usr/src/demo-rust-axum/data ./data

# SQLite数据目录（可挂载卷持久化）
ENV RUST_LOG=info
ENV SQLITE_PATH=/app/data/app.db

EXPOSE 3000

# 单个命令启动：API + 前端 + 数据库初始化
CMD ["./demo-rust-axum"]
