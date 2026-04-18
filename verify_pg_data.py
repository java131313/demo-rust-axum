#!/usr/bin/env python3
"""
验证 PostgreSQL 数据库中的 wubi_characters 数据
"""

import psycopg2

pg_config = {
    'host': 'localhost',
    'database': 'wubi_demo',
    'user': 'postgres',
    'password': 'password',
    'port': 5433
}

try:
    print("正在连接 PostgreSQL (端口 5433)...")
    pg_conn = psycopg2.connect(**pg_config)
    print("成功连接到 PostgreSQL")

    pg_cursor = pg_conn.cursor()

    pg_cursor.execute("SELECT COUNT(*) FROM wubi_characters")
    count = pg_cursor.fetchone()[0]
    print(f"\n数据库中共有 {count} 条记录")

    pg_cursor.execute("SELECT character_val, simple_code, full_code, pinyin FROM wubi_characters LIMIT 20")
    results = pg_cursor.fetchall()
    print("\n示例数据 (前20条):")
    for char, simple_code, full_code, pinyin in results:
        print(f"  {char}: 简码={simple_code} 全码={full_code} 拼音={pinyin}")

    pg_cursor.execute("SELECT COUNT(*) FROM wubi_characters WHERE pinyin != ''")
    with_pinyin = pg_cursor.fetchone()[0]
    print(f"\n有拼音的记录: {with_pinyin}/{count}")

    pg_cursor.close()
    pg_conn.close()
    print("\n数据库连接已关闭")

except psycopg2.Error as e:
    print(f"PostgreSQL 错误: {e}")