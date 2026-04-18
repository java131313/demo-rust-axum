# 多阶段构建Dockerfile - PostgreSQL版本
# 第一阶段：编译Rust应用
FROM rust:1.79-slim AS builder

WORKDIR /usr/src/demo-rust-axum

# 安装编译所需的依赖（包括PostgreSQL客户端库）
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# 复制依赖文件并预下载（利用Docker缓存层）
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# 复制完整源代码并编译
COPY . .
RUN touch src/main.rs && cargo build --release

# 第二阶段：创建最小化运行镜像
FROM debian:bookworm-slim

WORKDIR /app

# 安装运行时依赖（包括PostgreSQL客户端库）
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 从builder阶段复制编译好的二进制文件
COPY --from=builder /usr/src/demo-rust-axum/target/release/demo-rust-axum ./

# 复制数据文件（五笔字典）
COPY --from=builder /usr/src/demo-rust-axum/data ./data

# 设置环境变量
ENV RUST_LOG=info
ENV DATABASE_URL=postgresql://postgres:password@postgres:5432/wubi_demo

# 暴露端口
EXPOSE 3000

# 启动应用
CMD ["./demo-rust-axum", "0.0.0.0:3000"]
