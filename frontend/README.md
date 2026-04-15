# Wubi Typing Frontend

这是一个简单的 React + TypeScript 前端，用于连接 `demo-rust-axum` 后端的五笔教程接口。

## 运行

1. 进入前端目录：
   ```bash
   cd frontend
   ```
2. 安装依赖：
   ```bash
   npm install
   ```
3. 启动开发服务器：
   ```bash
   npm run dev
   ```

默认会在 `http://localhost:5173` 启动，所有 `/api/*` 请求将代理到 `http://127.0.0.1:3000`。
