#!/usr/bin/env python3
"""
从 MySQL 获取 wubi_characters 数据并生成 PostgreSQL SQL 脚本
"""

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

    mysql_conn = None

    try:
        # 连接 MySQL
        print("正在连接 MySQL...")
        mysql_conn = connect(**mysql_config)
        print("成功连接到 MySQL")
        
        # 读取 MySQL 数据
        mysql_cursor = mysql_conn.cursor()
        mysql_cursor.execute('SELECT character_val, wubi_code, pinyin FROM wubi_characters')
        rows = mysql_cursor.fetchall()
        print(f'从 MySQL 读取了 {len(rows)} 条数据')
        
        # 生成 SQL 脚本
        sql_content = """
-- PostgreSQL 更新脚本
-- 添加 pinyin 字段到 wubi_characters 表

ALTER TABLE wubi_characters ADD COLUMN IF NOT EXISTS pinyin VARCHAR(64) DEFAULT '';

-- 清空现有数据
TRUNCATE TABLE wubi_characters;

-- 插入数据
INSERT INTO wubi_characters (character_val, wubi_code, pinyin) VALUES
"""
        
        # 生成插入语句
        values = []
        for character_val, wubi_code, pinyin in rows:
            # 转义单引号
            character_val_escaped = character_val.replace("'", "''")
            pinyin_escaped = pinyin.replace("'", "''") if pinyin else "''"
            values.append(f"('{character_val_escaped}', '{wubi_code}', '{pinyin_escaped}')")
        
        # 拼接 SQL
        sql_content += ",\n".join(values)
        sql_content += "\nON CONFLICT (character_val) DO UPDATE SET\n    wubi_code = EXCLUDED.wubi_code,\n    pinyin = EXCLUDED.pinyin;

SELECT '数据导入完成! 共导入 ' || COUNT(*) || ' 条记录' AS message FROM wubi_characters;
"
        
        # 写入文件
        with open('wubi_characters_pg.sql', 'w', encoding='utf-8') as f:
            f.write(sql_content)
        
        print("成功生成 PostgreSQL SQL 脚本: wubi_characters_pg.sql")
        print(f"脚本包含 {len(rows)} 条数据")
        
    except Error as e:
        print(f"MySQL 错误: {e}")
    finally:
        if mysql_conn:
            mysql_conn.close()

if __name__ == "__main__":
    main()