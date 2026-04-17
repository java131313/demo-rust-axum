#!/usr/bin/env python3
"""
自动导入五笔编码数据到MySQL数据库
"""

import mysql.connector
from mysql.connector import Error
import re
import sys


def import_wubi_data():
    """导入五笔数据到数据库"""

    # 数据库配置
    db_config = {
        'host': '127.0.0.1',
        'database': 'wubi',
        'user': 'root',
        'password': 'sdsSDG123*^DD',
        'port': 3306
    }

    print("开始导入五笔编码数据...")

    # 连接数据库
    try:
        connection = mysql.connector.connect(**db_config)
        if connection.is_connected():
            print(f"成功连接到数据库 {db_config['database']}")
    except Error as e:
        print(f"数据库连接错误: {e}")
        return False

    try:
        cursor = connection.cursor()

        # 从SQL文件读取数据
        with open('wubi_data_complete.sql', 'r', encoding='utf-8') as f:
            sql_content = f.read()

        # 提取INSERT语句
        pattern = r'INSERT INTO wubi_characters \(character_val, wubi_code\) VALUES\s*(.*?);'
        match = re.search(pattern, sql_content, re.DOTALL)

        if not match:
            print("错误: 在SQL文件中未找到INSERT语句")
            return False

        values_text = match.group(1)
        value_pattern = r"\('([^']+)',\s*'([^']+)'\)"
        values = re.findall(value_pattern, values_text)

        print(f"从SQL文件中读取了 {len(values)} 条记录")

        # 准备插入语句（使用ON DUPLICATE KEY UPDATE）
        insert_query = """
        INSERT INTO wubi_characters (character_val, wubi_code)
        VALUES (%s, %s)
        ON DUPLICATE KEY UPDATE wubi_code = VALUES(wubi_code)
        """

        # 批量插入
        batch_size = 500
        total_processed = 0

        for i in range(0, len(values), batch_size):
            batch = values[i:i+batch_size]
            try:
                cursor.executemany(insert_query, batch)
                connection.commit()
                total_processed += len(batch)
                print(f"已处理批次 {i//batch_size + 1}: {len(batch)} 条记录")
            except Error as e:
                print(f"插入批次 {i//batch_size + 1} 时出错: {e}")
                # 逐条插入
                for char, code in batch:
                    try:
                        cursor.execute(insert_query, (char, code))
                    except Error as e2:
                        print(f"  跳过问题数据: '{char}' -> '{code}', 错误: {e2}")
                connection.commit()

        # 统计结果
        cursor.execute("SELECT COUNT(*) FROM wubi_characters")
        total_in_db = cursor.fetchone()[0]

        print(f"\n数据导入完成!")
        print(f"  处理记录数: {len(values)}")
        print(f"  数据库中现有: {total_in_db} 条记录")

        # 显示一些样本
        cursor.execute("SELECT character_val, wubi_code FROM wubi_characters ORDER BY character_val LIMIT 10")
        sample = cursor.fetchall()

        print(f"\n数据库中的前10条记录:")
        for char, code in sample:
            print(f"  {char} -> {code}")

        cursor.close()
        return True

    except Error as e:
        print(f"导入数据时出错: {e}")
        return False
    finally:
        if connection and connection.is_connected():
            connection.close()
            print("数据库连接已关闭")


def main():
    """主函数"""
    print("=" * 60)
    print("五笔编码数据自动导入工具")
    print("=" * 60)

    success = import_wubi_data()

    if success:
        print("\n" + "=" * 60)
        print("导入成功!")
        print("=" * 60)
    else:
        print("\n" + "=" * 60)
        print("导入失败!")
        print("=" * 60)
        sys.exit(1)


if __name__ == "__main__":
    main()