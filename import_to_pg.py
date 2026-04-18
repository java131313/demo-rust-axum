#!/usr/bin/env python3
"""
将 wubi86_pg.sql 导入到 PostgreSQL 数据库
"""

import psycopg2

pg_config = {
    'host': 'localhost',
    'database': 'wubi_demo',
    'user': 'postgres',
    'password': 'password',
    'port': 5432
}

try:
    print("正在连接 PostgreSQL...")
    pg_conn = psycopg2.connect(**pg_config)
    print("成功连接到 PostgreSQL")

    with open('wubi86_pg.sql', 'r', encoding='utf-8') as f:
        sql_content = f.read()

    print("正在执行 SQL 脚本...")
    pg_cursor = pg_conn.cursor()
    pg_cursor.execute(sql_content)

    pg_conn.commit()

    pg_cursor.execute("SELECT COUNT(*) FROM wubi_characters")
    count = pg_cursor.fetchone()[0]
    print(f"\n成功! 数据库中现在有 {count} 条记录")

    pg_cursor.execute("SELECT character_val, wubi_code, pinyin FROM wubi_characters LIMIT 20")
    results = pg_cursor.fetchall()
    print("\n示例数据:")
    for char, code, pinyin in results:
        print(f"  {char}: {code} - {pinyin}")

    pg_cursor.close()
    pg_conn.close()
    print("\n数据库连接已关闭")

except psycopg2.Error as e:
    print(f"PostgreSQL 错误: {e}")
    print("\n可能的解决方案:")
    print("1. 确保 PostgreSQL 服务正在运行")
    print("2. 检查 docker-compose up -d postgres 是否启动")
    print("3. 或者手动执行: psql -U postgres -d wubi_demo -f wubi86_pg.sql")