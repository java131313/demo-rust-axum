#!/usr/bin/env python3
"""
检查 PostgreSQL 数据库中 wubi_characters 表的结构
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

    print("\n查询 wubi_characters 表结构...")
    pg_cursor.execute("""
        SELECT column_name, data_type, character_maximum_length, is_nullable
        FROM information_schema.columns
        WHERE table_name = 'wubi_characters'
        ORDER BY ordinal_position
    """)
    columns = pg_cursor.fetchall()

    print("\n表结构:")
    for col in columns:
        print(f"  {col[0]}: {col[1]}{'(' + str(col[2]) + ')' if col[2] else ''} {'NULL' if col[3] == 'YES' else 'NOT NULL'}")

    print("\n查询表中的数据...")
    pg_cursor.execute("SELECT COUNT(*) FROM wubi_characters")
    count = pg_cursor.fetchone()[0]
    print(f"表中现有 {count} 条记录")

    if count > 0:
        print("\n前10条数据:")
        pg_cursor.execute("SELECT * FROM wubi_characters LIMIT 10")
        results = pg_cursor.fetchall()
        for row in results:
            print(f"  {row}")

    pg_cursor.close()
    pg_conn.close()

except psycopg2.Error as e:
    print(f"PostgreSQL 错误: {e}")