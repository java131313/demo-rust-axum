#!/usr/bin/env python3
"""
更新wubi_characters表的pinyin字段
使用pypinyin库获取汉字拼音
"""

import mysql.connector
from mysql.connector import Error
from pypinyin import pinyin, Style


db_config = {
    'host': '127.0.0.1',
    'database': 'wubi',
    'user': 'root',
    'password': 'sdsSDG123*^DD',
    'port': 3306
}

def get_pinyin(char):
    """获取汉字拼音"""
    try:
        result = pinyin(char, style=Style.TONE, heteronym=False)
        if result and result[0]:
            return result[0][0]
    except Exception as e:
        print(f"Error getting pinyin for {char}: {e}")
    return ""

def update_pinyin():
    """更新所有字符的拼音"""
    try:
        connection = mysql.connector.connect(**db_config)
        if connection.is_connected():
            print(f"成功连接到数据库 {db_config['database']}")
    except Error as e:
        print(f"数据库连接错误: {e}")
        return False

    try:
        cursor = connection.cursor()

        cursor.execute("SELECT id, character_val FROM wubi_characters")
        characters = cursor.fetchall()

        total = len(characters)
        print(f"总共需要更新 {total} 个字符")

        updated = 0
        for idx, (id, char) in enumerate(characters):
            pinyin_val = get_pinyin(char)
            if pinyin_val:
                cursor.execute(
                    "UPDATE wubi_characters SET pinyin = %s WHERE id = %s",
                    (pinyin_val, id)
                )
                updated += 1

            if (idx + 1) % 50 == 0:
                print(f"进度: {idx + 1}/{total}")
                connection.commit()

        connection.commit()
        print(f"更新完成! 成功更新 {updated}/{total} 个字符的拼音")

        cursor.execute("SELECT character_val, pinyin FROM wubi_characters WHERE pinyin != '' LIMIT 20")
        results = cursor.fetchall()
        print("\n更新后的示例数据:")
        for char, pinyin_val in results:
            print(f"  {char}: {pinyin_val}")

        return True

    except Error as e:
        print(f"数据库错误: {e}")
        return False
    finally:
        if connection.is_connected():
            cursor.close()
            connection.close()
            print("数据库连接已关闭")

if __name__ == "__main__":
    update_pinyin()