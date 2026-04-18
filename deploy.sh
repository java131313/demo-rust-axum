#!/bin/bash
# Docker部署脚本

set -e

echo "======================================"
echo "五笔打字练习 - Docker部署脚本"
echo "======================================"

# 检查Docker是否安装
if ! command -v docker &> /dev/null; then
    echo "错误: Docker未安装，请先安装Docker"
    exit 1
fi

if ! command -v docker compose &> /dev/null; then
    echo "错误: Docker Compose未安装"
    exit 1
fi

echo ""
echo "步骤 1/4: 停止现有容器..."
docker compose down || true

echo ""
echo "步骤 2/4: 构建Docker镜像..."
docker compose build

echo ""
echo "步骤 3/4: 启动服务..."
docker compose up -d

echo ""
echo "步骤 4/4: 等待服务启动..."
sleep 10

echo ""
echo "======================================"
echo "部署完成！"
echo "======================================"
echo ""
echo "服务访问地址："
echo "  - 后端API: http://localhost:3000"
echo "  - 前端应用: http://localhost:5173"
echo "  - MySQL数据库: localhost:3306"
echo ""
echo "查看日志: docker compose logs -f"
echo "停止服务: docker compose down"
echo "重启服务: docker compose restart"
echo ""
echo "Docker容器状态:"
docker compose ps
