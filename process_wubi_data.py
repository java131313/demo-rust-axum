#!/usr/bin/env python3
"""
处理现有五笔编码数据并插入到MySQL数据库
从多个来源整合数据
"""

import mysql.connector
from mysql.connector import Error
import re
import os
import json
from typing import List, Tuple


class WubiDataProcessor:
    def __init__(self):
        # 数据库配置
        self.db_config = {
            'host': '127.0.0.1',
            'database': 'wubi',
            'user': 'root',
            'password': 'sdsSDG123*^DD',
            'port': 3306
        }

        # 存储所有数据
        self.wubi_data = []

    def extract_data_from_py_file(self, filename: str = "import_wubi_data.py") -> List[Tuple[str, str]]:
        """
        从Python文件中提取wubi_data列表
        """
        data = []

        try:
            with open(filename, 'r', encoding='utf-8') as f:
                content = f.read()

            # 查找wubi_data列表
            pattern = r'wubi_data\s*=\s*\[(.*?)\]'
            match = re.search(pattern, content, re.DOTALL)

            if match:
                data_text = match.group(1)
                # 提取所有元组
                tuple_pattern = r'\("([^"]+)",\s*"([^"]+)"\)'
                tuples = re.findall(tuple_pattern, data_text)

                for char, code in tuples:
                    data.append((char.strip(), code.strip()))

                print(f"从 {filename} 中提取了 {len(data)} 条记录")
            else:
                print(f"在 {filename} 中未找到 wubi_data 列表")

        except Exception as e:
            print(f"读取文件 {filename} 时出错: {e}")

        return data

    def extract_data_from_rust_file(self, filename: str = "src/wubi.rs") -> List[Tuple[str, str]]:
        """
        从Rust文件中提取wubi数据
        """
        data = []

        try:
            with open(filename, 'r', encoding='utf-8') as f:
                content = f.read()

            # 查找Rust数组
            pattern = r'let rows\s*=\s*\[(.*?)\];'
            match = re.search(pattern, content, re.DOTALL)

            if match:
                data_text = match.group(1)
                # 提取所有元组，格式: ("王", "gggg")
                tuple_pattern = r'\("([^"]+)",\s*"([^"]+)"\)'
                tuples = re.findall(tuple_pattern, data_text)

                for char, code in tuples:
                    data.append((char.strip(), code.strip()))

                print(f"从 {filename} 中提取了 {len(data)} 条记录")
            else:
                print(f"在 {filename} 中未找到 wubi 数据数组")

        except Exception as e:
            print(f"读取文件 {filename} 时出错: {e}")

        return data

    def load_data_from_sql_file(self, filename: str = "insert_wubi_characters.sql") -> List[Tuple[str, str]]:
        """
        从SQL文件中提取数据
        """
        data = []

        try:
            with open(filename, 'r', encoding='utf-8') as f:
                content = f.read()

            # 查找INSERT语句
            pattern = r'INSERT INTO wubi_characters \(character_val, wubi_code\) VALUES\s*(.*?);'
            match = re.search(pattern, content, re.DOTALL)

            if match:
                values_text = match.group(1)
                # 提取所有值对，格式: ('王', 'gggg')
                value_pattern = r"\('([^']+)',\s*'([^']+)'\)"
                values = re.findall(value_pattern, values_text)

                for char, code in values:
                    data.append((char.strip(), code.strip()))

                print(f"从 {filename} 中提取了 {len(data)} 条记录")
            else:
                print(f"在 {filename} 中未找到 INSERT 语句")

        except Exception as e:
            print(f"读取文件 {filename} 时出错: {e}")

        return data

    def collect_all_data(self):
        """
        从所有可用来源收集数据
        """
        print("开始收集五笔编码数据...")

        # 从各个文件收集数据
        all_data = {}

        # 1. 从Python文件
        py_data = self.extract_data_from_py_file("import_wubi_data.py")
        for char, code in py_data:
            all_data[char] = code

        # 2. 从Rust文件
        rust_data = self.extract_data_from_rust_file("src/wubi.rs")
        for char, code in rust_data:
            if char not in all_data:  # 只添加新的
                all_data[char] = code
            elif len(code) > len(all_data[char]):  # 如果新编码更长，使用新编码
                all_data[char] = code

        # 3. 从SQL文件
        sql_data = self.load_data_from_sql_file("insert_wubi_characters.sql")
        for char, code in sql_data:
            if char not in all_data:
                all_data[char] = code
            elif len(code) > len(all_data[char]):
                all_data[char] = code

        # 转换为列表并排序
        self.wubi_data = [(char, code) for char, code in all_data.items()]
        self.wubi_data.sort(key=lambda x: x[0])  # 按汉字排序

        print(f"总共收集到 {len(self.wubi_data)} 条唯一记录")

    def connect_database(self):
        """
        连接MySQL数据库
        """
        try:
            connection = mysql.connector.connect(
                host=self.db_config['host'],
                database=self.db_config['database'],
                user=self.db_config['user'],
                password=self.db_config['password'],
                port=self.db_config['port']
            )

            if connection.is_connected():
                print(f"成功连接到数据库 {self.db_config['database']}")
                return connection

        except Error as e:
            print(f"数据库连接错误: {e}")
            return None

    def clear_existing_data(self, connection):
        """
        清空现有数据（可选）
        """
        try:
            cursor = connection.cursor()
            cursor.execute("TRUNCATE TABLE wubi_characters")
            connection.commit()
            print("已清空 wubi_characters 表")
            cursor.close()
            return True
        except Error as e:
            print(f"清空数据时出错: {e}")
            return False

    def import_to_database(self, clear_existing: bool = False):
        """
        导入数据到数据库
        """
        if not self.wubi_data:
            print("错误: 没有数据可导入!")
            return False

        connection = self.connect_database()
        if not connection:
            print("无法连接数据库")
            return False

        try:
            cursor = connection.cursor()

            # 如果需要，清空现有数据
            if clear_existing:
                self.clear_existing_data(connection)

            # 准备插入语句（使用ON DUPLICATE KEY UPDATE处理重复）
            insert_query = """
            INSERT INTO wubi_characters (character_val, wubi_code)
            VALUES (%s, %s)
            ON DUPLICATE KEY UPDATE wubi_code = VALUES(wubi_code)
            """

            # 批量插入
            batch_size = 1000
            total_inserted = 0

            for i in range(0, len(self.wubi_data), batch_size):
                batch = self.wubi_data[i:i+batch_size]
                try:
                    cursor.executemany(insert_query, batch)
                    connection.commit()
                    total_inserted += len(batch)
                    print(f"已插入批次 {i//batch_size + 1}: {len(batch)} 条记录")
                except Error as e:
                    print(f"插入批次 {i//batch_size + 1} 时出错: {e}")
                    # 逐条插入以找出问题数据
                    for char, code in batch:
                        try:
                            cursor.execute(insert_query, (char, code))
                        except Error as e2:
                            print(f"  跳过问题数据: '{char}' -> '{code}', 错误: {e2}")
                    connection.commit()

            # 统计数据库中的总记录数
            cursor.execute("SELECT COUNT(*) FROM wubi_characters")
            total_in_db = cursor.fetchone()[0]

            print(f"\n数据导入完成!")
            print(f"  尝试导入: {len(self.wubi_data)} 条记录")
            print(f"  数据库中现有: {total_in_db} 条记录")

            # 显示一些统计信息
            cursor.execute("SELECT MIN(character_val), MAX(character_val) FROM wubi_characters")
            min_char, max_char = cursor.fetchone()
            print(f"  汉字范围: {min_char} - {max_char}")

            cursor.execute("SELECT COUNT(DISTINCT wubi_code) FROM wubi_characters")
            unique_codes = cursor.fetchone()[0]
            print(f"  唯一五笔编码数: {unique_codes}")

            cursor.close()
            return True

        except Error as e:
            print(f"导入数据时出错: {e}")
            return False

        finally:
            if connection and connection.is_connected():
                connection.close()
                print("数据库连接已关闭")

    def export_to_files(self):
        """
        导出数据到文件
        """
        if not self.wubi_data:
            print("错误: 没有数据可导出!")
            return False

        try:
            # 导出到SQL文件
            sql_filename = "wubi_data_complete.sql"
            with open(sql_filename, 'w', encoding='utf-8') as f:
                f.write("-- 五笔编码完整数据插入脚本\n")
                f.write(f"-- 生成时间: 2026-04-17\n")
                f.write(f"-- 数据量: {len(self.wubi_data)} 条记录\n\n")

                f.write("INSERT INTO wubi_characters (character_val, wubi_code) VALUES\n")

                for i, (character, code) in enumerate(self.wubi_data):
                    if i > 0:
                        f.write(",\n")
                    f.write(f"  ('{character}', '{code}')")

                f.write(";\n")

            print(f"数据已导出到SQL文件: {sql_filename}")

            # 导出到CSV文件
            csv_filename = "wubi_data_complete.csv"
            import csv
            with open(csv_filename, 'w', encoding='utf-8', newline='') as f:
                writer = csv.writer(f)
                writer.writerow(['character', 'wubi_code'])
                for character, code in self.wubi_data:
                    writer.writerow([character, code])

            print(f"数据已导出到CSV文件: {csv_filename}")

            # 导出到Rust数组格式
            rust_filename = "wubi_data_rust.txt"
            with open(rust_filename, 'w', encoding='utf-8') as f:
                f.write("// 五笔编码数据 - Rust数组格式\n")
                f.write(f"// 数据量: {len(self.wubi_data)} 条记录\n\n")
                f.write("let wubi_data = &[\n")

                for i, (character, code) in enumerate(self.wubi_data):
                    if i % 10 == 0 and i > 0:
                        f.write("\n")
                    f.write(f'    ("{character}", "{code}"),\n')

                f.write("];\n")

            print(f"数据已导出到Rust格式文件: {rust_filename}")

            return True

        except Exception as e:
            print(f"导出文件时出错: {e}")
            return False

    def show_sample_data(self, count: int = 20):
        """
        显示样本数据
        """
        if not self.wubi_data:
            print("没有数据可显示")
            return

        print(f"\n前 {count} 条样本数据:")
        print("-" * 40)
        for i, (char, code) in enumerate(self.wubi_data[:count]):
            print(f"{i+1:3d}. {char} -> {code}")
        print("-" * 40)

        # 显示一些统计信息
        print(f"\n数据统计:")
        print(f"  总记录数: {len(self.wubi_data)}")

        # 计算编码长度分布
        code_lengths = {}
        for char, code in self.wubi_data:
            length = len(code)
            code_lengths[length] = code_lengths.get(length, 0) + 1

        print(f"  编码长度分布:")
        for length in sorted(code_lengths.keys()):
            print(f"    {length} 字符: {code_lengths[length]} 条")


def main():
    """主函数"""
    print("=" * 60)
    print("五笔编码数据处理工具")
    print("=" * 60)

    # 创建处理器
    processor = WubiDataProcessor()

    # 收集数据
    processor.collect_all_data()

    if not processor.wubi_data:
        print("错误: 无法收集到任何数据!")
        return

    # 显示样本数据
    processor.show_sample_data(20)

    # 导出数据到文件
    print("\n导出数据到文件...")
    processor.export_to_files()

    # 询问是否导入数据库
    print("\n" + "=" * 60)
    response = input("是否导入数据到MySQL数据库? (y/n): ")

    if response.lower() == 'y':
        clear_response = input("是否清空现有数据? (y/n): ")
        clear_existing = (clear_response.lower() == 'y')

        print("\n开始导入数据到数据库...")
        success = processor.import_to_database(clear_existing=clear_existing)

        if success:
            print("\n" + "=" * 60)
            print("数据导入完成!")
            print("=" * 60)
        else:
            print("\n" + "=" * 60)
            print("数据导入失败!")
            print("=" * 60)
    else:
        print("\n跳过数据库导入")
        print("您可以使用以下命令手动导入:")
        print("  mysql -u root -p wubi < wubi_data_complete.sql")

    print("\n处理完成!")


if __name__ == "__main__":
    main()