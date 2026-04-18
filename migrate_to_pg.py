#!/usr/bin/env python3
"""
将 MySQL 中的 wubi_characters 数据迁移到 PostgreSQL
"""

import psycopg2
from mysql.connector import connect, Error

def main():
    # 连接 MySQL
    mysql_config = {
        'host': '127.0.0.1',
        'database': 'wubi',
        'user': 'root',
        'password': 'sdsSDG123*^DD',
        'port': 3306
    }

    # 连接 PostgreSQL
    pg_config = {
        'host': 'localhost',
        'database': 'wubi_demo',
        'user': 'postgres',
        'password': 'password',
        'port': 5432
    }

    mysql_conn = None
    pg_conn = None

    try:
        # 连接 MySQL
        print("正在连接 MySQL...")
        mysql_conn = connect(**mysql_config)
        print("成功连接到 MySQL")
        
        # 连接 PostgreSQL
        print("正在连接 PostgreSQL...")
        pg_conn = psycopg2.connect(**pg_config)
        print("成功连接到 PostgreSQL")
        
        # 读取 MySQL 数据
        mysql_cursor = mysql_conn.cursor()
        mysql_cursor.execute('SELECT character_val, wubi_code, pinyin FROM wubi_characters')
        rows = mysql_cursor.fetchall()
        print(f'从 MySQL 读取了 {len(rows)} 条数据')
        
        # 插入到 PostgreSQL
        pg_cursor = pg_conn.cursor()
        
        # 先清空 PostgreSQL 表
        print("正在清空 PostgreSQL 表...")
        pg_cursor.execute('TRUNCATE TABLE wubi_characters')
        
        # 插入数据
        print("正在插入数据到 PostgreSQL...")
        inserted = 0
        for character_val, wubi_code, pinyin in rows:
            pg_cursor.execute(
                'INSERT INTO wubi_characters (character_val, wubi_code, pinyin) VALUES (%s, %s, %s)',
                (character_val, wubi_code, pinyin)
            )
            inserted += 1
        
        pg_conn.commit()
        print(f'成功插入 {inserted} 条数据到 PostgreSQL')
        
        # 验证数据
        pg_cursor.execute('SELECT COUNT(*) FROM wubi_characters')
        count = pg_cursor.fetchone()[0]
        print(f'PostgreSQL 表中现在有 {count} 条数据')
        
        # 显示前10条数据
        pg_cursor.execute('SELECT character_val, wubi_code, pinyin FROM wubi_characters LIMIT 10')
        sample = pg_cursor.fetchall()
        print('示例数据:')
        for row in sample:
            print(row)
            
    except Error as e:
        print(f"MySQL 错误: {e}")
    except psycopg2.Error as e:
        print(f"PostgreSQL 错误: {e}")
    finally:
        if mysql_conn:
            mysql_conn.close()
        if pg_conn:
            pg_conn.close()

if __name__ == "__main__":
    main()