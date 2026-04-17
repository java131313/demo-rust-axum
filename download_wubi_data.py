#!/usr/bin/env python3
"""
从网上下载新华字典汉字和五笔编码数据并导入到MySQL数据库
支持多个数据源，自动下载并导入
"""

import os
import sys
import csv
import json
import sqlite3
import requests
import mysql.connector
from mysql.connector import Error
from typing import List, Tuple, Dict, Optional
import hashlib
import time
import re


class WubiDataDownloader:
    def __init__(self, db_config=None):
        """
        初始化下载器

        Args:
            db_config: 数据库配置字典，包含host, database, user, password等
        """
        if db_config is None:
            # 默认数据库配置，从Rust代码中获取
            self.db_config = {
                'host': '127.0.0.1',
                'database': 'wubi',
                'user': 'root',
                'password': 'sdsSDG123*^DD',
                'port': 3306
            }
        else:
            self.db_config = db_config

        # 数据源列表
        self.data_sources = [
            {
                'name': 'GitHub - fengyhack/wubi',
                'url': 'https://raw.githubusercontent.com/fengyhack/wubi/master/data/wubi86.txt',
                'type': 'text',
                'parser': self.parse_wubi86_text
            },
            {
                'name': '开源五笔码表',
                'url': 'https://raw.githubusercontent.com/studyzy/imewlconverter/master/Windows/五笔86.txt',
                'type': 'text',
                'parser': self.parse_imewlconverter_text
            },
            {
                'name': 'Rime五笔',
                'url': 'https://raw.githubusercontent.com/rime/rime-wubi/master/wubi86.dict.yaml',
                'type': 'yaml',
                'parser': self.parse_rime_yaml
            }
        ]

        # 备用本地数据文件
        self.local_data_files = [
            'wubi_data.csv',
            'wubi86.txt',
            '五笔86.txt'
        ]

    def download_from_url(self, url: str, timeout: int = 30) -> Optional[str]:
        """
        从URL下载数据

        Args:
            url: 下载地址
            timeout: 超时时间

        Returns:
            下载的文本内容或None
        """
        try:
            print(f"正在从 {url} 下载数据...")
            headers = {
                'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
            }
            response = requests.get(url, headers=headers, timeout=timeout)
            response.raise_for_status()

            # 检测编码
            if response.encoding is None:
                response.encoding = 'utf-8'

            print(f"下载成功，大小: {len(response.content)} 字节")
            return response.text

        except requests.exceptions.RequestException as e:
            print(f"下载失败: {e}")
            return None
        except Exception as e:
            print(f"处理下载数据时出错: {e}")
            return None

    def parse_wubi86_text(self, content: str) -> List[Tuple[str, str]]:
        """
        解析wubi86.txt格式的数据

        格式示例:
        的 rqyy
        一 g
        是 jghu
        """
        data = []
        lines = content.split('\n')

        for line in lines:
            line = line.strip()
            if not line or line.startswith('#'):
                continue

            # 尝试多种分隔符
            parts = None
            if '\t' in line:
                parts = line.split('\t')
            elif ' ' in line:
                parts = line.split(' ')

            if parts and len(parts) >= 2:
                character = parts[0].strip()
                code = parts[1].strip()
                if character and code:
                    data.append((character, code))

        print(f"从wubi86格式解析出 {len(data)} 条记录")
        return data

    def parse_imewlconverter_text(self, content: str) -> List[Tuple[str, str]]:
        """
        解析imewlconverter格式的数据

        格式示例:
        a 工
        aa 式
        aaaa 工
        """
        data = []
        lines = content.split('\n')

        for line in lines:
            line = line.strip()
            if not line:
                continue

            parts = line.split()
            if len(parts) >= 2:
                code = parts[0].strip()
                character = parts[1].strip()
                if character and code:
                    data.append((character, code))

        print(f"从imewlconverter格式解析出 {len(data)} 条记录")
        return data

    def parse_rime_yaml(self, content: str) -> List[Tuple[str, str]]:
        """
        解析Rime YAML格式的数据

        格式示例:
        的	rqyy
        一	g
        是	jghu
        """
        data = []
        lines = content.split('\n')
        in_dict_section = False

        for line in lines:
            line = line.strip()
            if not line:
                continue

            # 查找字典部分开始
            if line.startswith('...') or line.startswith('---'):
                continue

            if ':' in line and 'name' in line.lower():
                in_dict_section = True
                continue

            if in_dict_section:
                # 跳过注释
                if line.startswith('#'):
                    continue

                # 解析键值对
                if '\t' in line:
                    parts = line.split('\t')
                    if len(parts) >= 2:
                        character = parts[0].strip()
                        code = parts[1].strip().split()[0] if ' ' in parts[1] else parts[1].strip()
                        if character and code:
                            data.append((character, code))

        print(f"从Rime YAML格式解析出 {len(data)} 条记录")
        return data

    def check_local_data_files(self) -> Optional[List[Tuple[str, str]]]:
        """
        检查本地数据文件

        Returns:
            从本地文件解析的数据或None
        """
        for filename in self.local_data_files:
            if os.path.exists(filename):
                print(f"发现本地数据文件: {filename}")
                try:
                    with open(filename, 'r', encoding='utf-8') as f:
                        content = f.read()

                    # 根据文件扩展名选择解析器
                    if filename.endswith('.csv'):
                        return self.parse_csv(content)
                    elif filename.endswith('.txt'):
                        # 尝试多种解析器
                        for parser in [self.parse_wubi86_text, self.parse_imewlconverter_text]:
                            try:
                                data = parser(content)
                                if data:
                                    return data
                            except:
                                continue
                except Exception as e:
                    print(f"读取本地文件 {filename} 时出错: {e}")

        return None

    def parse_csv(self, content: str) -> List[Tuple[str, str]]:
        """
        解析CSV格式的数据
        """
        data = []
        lines = content.split('\n')

        for line in csv.reader(lines):
            if len(line) >= 2:
                character = line[0].strip()
                code = line[1].strip()
                if character and code:
                    data.append((character, code))

        print(f"从CSV格式解析出 {len(data)} 条记录")
        return data

    def download_all_sources(self) -> List[Tuple[str, str]]:
        """
        尝试从所有数据源下载数据

        Returns:
            合并后的数据列表
        """
        all_data = {}

        # 先检查本地文件
        local_data = self.check_local_data_files()
        if local_data:
            for character, code in local_data:
                all_data[character] = code
            print(f"从本地文件加载 {len(local_data)} 条记录")

        # 尝试在线数据源
        for source in self.data_sources:
            print(f"\n尝试数据源: {source['name']}")
            content = self.download_from_url(source['url'])
            if content:
                try:
                    data = source['parser'](content)
                    for character, code in data:
                        # 只添加新的或覆盖更长的编码（更完整）
                        if character not in all_data or len(code) > len(all_data[character]):
                            all_data[character] = code
                    print(f"从 {source['name']} 添加了 {len(data)} 条记录")
                except Exception as e:
                    print(f"解析 {source['name']} 数据时出错: {e}")
            else:
                print(f"无法从 {source['name']} 下载数据")

            # 避免请求过快
            time.sleep(1)

        # 转换为列表
        result = [(char, code) for char, code in all_data.items()]
        print(f"\n总共收集到 {len(result)} 条唯一记录")
        return result

    def connect_database(self) -> Optional[mysql.connector.MySQLConnection]:
        """
        连接MySQL数据库

        Returns:
            数据库连接对象或None
        """
        try:
            connection = mysql.connector.connect(
                host=self.db_config['host'],
                database=self.db_config['database'],
                user=self.db_config['user'],
                password=self.db_config['password'],
                port=self.db_config.get('port', 3306)
            )

            if connection.is_connected():
                print(f"成功连接到数据库 {self.db_config['database']}")
                return connection

        except Error as e:
            print(f"数据库连接错误: {e}")
            print("请检查数据库配置:")
            print(f"  主机: {self.db_config['host']}")
            print(f"  数据库: {self.db_config['database']}")
            print(f"  用户: {self.db_config['user']}")
            print(f"  端口: {self.db_config.get('port', 3306)}")

            # 尝试创建数据库
            if "Unknown database" in str(e):
                print("尝试创建数据库...")
                try:
                    # 连接时不指定数据库
                    temp_conn = mysql.connector.connect(
                        host=self.db_config['host'],
                        user=self.db_config['user'],
                        password=self.db_config['password'],
                        port=self.db_config.get('port', 3306)
                    )
                    cursor = temp_conn.cursor()
                    cursor.execute(f"CREATE DATABASE IF NOT EXISTS {self.db_config['database']} CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci")
                    temp_conn.commit()
                    cursor.close()
                    temp_conn.close()
                    print(f"数据库 {self.db_config['database']} 创建成功")

                    # 重新连接
                    return self.connect_database()
                except Error as e2:
                    print(f"创建数据库失败: {e2}")

        return None

    def create_table(self, connection: mysql.connector.MySQLConnection):
        """
        创建数据表（如果不存在）
        """
        try:
            cursor = connection.cursor()

            create_table_query = """
            CREATE TABLE IF NOT EXISTS wubi_characters (
                id INT AUTO_INCREMENT PRIMARY KEY,
                character_val VARCHAR(4) NOT NULL UNIQUE,
                wubi_code VARCHAR(8) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                INDEX idx_character (character_val),
                INDEX idx_code (wubi_code)
            ) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci
            """

            cursor.execute(create_table_query)
            connection.commit()
            print("数据表 wubi_characters 已创建或已存在")
            cursor.close()

        except Error as e:
            print(f"创建数据表时出错: {e}")
            raise

    def import_data(self, data: List[Tuple[str, str]], batch_size: int = 1000):
        """
        导入数据到数据库

        Args:
            data: 数据列表，每个元素为(汉字, 五笔编码)
            batch_size: 批量插入的大小
        """
        connection = self.connect_database()
        if not connection:
            print("无法连接数据库，跳过数据导入")
            return False

        try:
            self.create_table(connection)
            cursor = connection.cursor()

            # 使用ON DUPLICATE KEY UPDATE避免重复
            insert_query = """
            INSERT INTO wubi_characters (character_val, wubi_code)
            VALUES (%s, %s)
            ON DUPLICATE KEY UPDATE wubi_code = VALUES(wubi_code)
            """

            total_inserted = 0
            total_updated = 0

            # 批量插入
            for i in range(0, len(data), batch_size):
                batch = data[i:i+batch_size]
                try:
                    cursor.executemany(insert_query, batch)
                    connection.commit()

                    # 获取影响的行数
                    affected = cursor.rowcount
                    # 在ON DUPLICATE KEY UPDATE情况下，rowcount可能不准确
                    # 我们简单估计一下
                    total_inserted += len(batch)

                    print(f"已处理批次 {i//batch_size + 1}: {len(batch)} 条记录")

                except Error as e:
                    print(f"插入批次 {i//batch_size + 1} 时出错: {e}")
                    # 尝试逐条插入以找出问题数据
                    for char, code in batch:
                        try:
                            cursor.execute(insert_query, (char, code))
                        except Error as e2:
                            print(f"  跳过问题数据: '{char}' -> '{code}', 错误: {e2}")
                    connection.commit()

            # 统计实际数据量
            cursor.execute("SELECT COUNT(*) FROM wubi_characters")
            total_in_db = cursor.fetchone()[0]

            print(f"\n数据导入完成!")
            print(f"  尝试导入: {len(data)} 条记录")
            print(f"  数据库中现有: {total_in_db} 条记录")

            cursor.close()
            return True

        except Error as e:
            print(f"导入数据时出错: {e}")
            return False

        finally:
            if connection and connection.is_connected():
                connection.close()
                print("数据库连接已关闭")

    def export_to_sql_file(self, data: List[Tuple[str, str]], filename: str = "wubi_data_insert.sql"):
        """
        导出数据为SQL文件

        Args:
            data: 数据列表
            filename: 输出文件名
        """
        try:
            with open(filename, 'w', encoding='utf-8') as f:
                f.write("-- 五笔编码数据插入脚本\n")
                f.write("-- 生成时间: " + time.strftime("%Y-%m-%d %H:%M:%S") + "\n")
                f.write(f"-- 数据量: {len(data)} 条记录\n\n")

                f.write("INSERT INTO wubi_characters (character_val, wubi_code) VALUES\n")

                for i, (character, code) in enumerate(data):
                    if i > 0:
                        f.write(",\n")
                    f.write(f"  ('{character}', '{code}')")

                f.write(";\n\n")
                f.write("-- 结束\n")

            print(f"数据已导出到文件: {filename}")
            return True

        except Exception as e:
            print(f"导出SQL文件时出错: {e}")
            return False

    def export_to_csv(self, data: List[Tuple[str, str]], filename: str = "wubi_data.csv"):
        """
        导出数据为CSV文件

        Args:
            data: 数据列表
            filename: 输出文件名
        """
        try:
            with open(filename, 'w', encoding='utf-8', newline='') as f:
                writer = csv.writer(f)
                writer.writerow(['character', 'wubi_code'])
                for character, code in data:
                    writer.writerow([character, code])

            print(f"数据已导出到CSV文件: {filename}")
            return True

        except Exception as e:
            print(f"导出CSV文件时出错: {e}")
            return False

    def run(self, export_only: bool = False):
        """
        运行下载和导入流程

        Args:
            export_only: 仅导出数据，不导入数据库
        """
        print("=" * 60)
        print("五笔编码数据下载工具")
        print("=" * 60)

        # 下载数据
        print("\n1. 下载数据...")
        data = self.download_all_sources()

        if not data:
            print("错误: 无法获取任何数据!")
            return False

        # 去重和排序
        data_dict = {}
        for character, code in data:
            if character not in data_dict:
                data_dict[character] = code
            elif len(code) > len(data_dict[character]):
                # 保留更长的编码（通常更完整）
                data_dict[character] = code

        data = [(char, code) for char, code in data_dict.items()]
        data.sort(key=lambda x: x[0])  # 按汉字排序

        print(f"\n2. 数据处理完成，共 {len(data)} 条唯一记录")

        # 显示样本数据
        print("\n样本数据（前10条）:")
        for i, (char, code) in enumerate(data[:10]):
            print(f"  {i+1:2d}. {char} -> {code}")

        # 导出数据
        print("\n3. 导出数据...")
        self.export_to_sql_file(data, "wubi_data_insert.sql")
        self.export_to_csv(data, "wubi_data.csv")

        if export_only:
            print("\n仅导出模式，跳过数据库导入")
            return True

        # 导入数据库
        print("\n4. 导入数据库...")
        success = self.import_data(data)

        if success:
            print("\n" + "=" * 60)
            print("数据下载和导入完成!")
            print("=" * 60)
            return True
        else:
            print("\n" + "=" * 60)
            print("数据导入失败!")
            print("=" * 60)
            return False


def main():
    """主函数"""
    import argparse

    parser = argparse.ArgumentParser(description='下载五笔编码数据并导入数据库')
    parser.add_argument('--export-only', action='store_true', help='仅导出数据，不导入数据库')
    parser.add_argument('--host', default='127.0.0.1', help='MySQL主机地址')
    parser.add_argument('--database', default='wubi', help='数据库名称')
    parser.add_argument('--user', default='root', help='数据库用户')
    parser.add_argument('--password', default='sdsSDG123*^DD', help='数据库密码')
    parser.add_argument('--port', type=int, default=3306, help='数据库端口')

    args = parser.parse_args()

    # 配置数据库
    db_config = {
        'host': args.host,
        'database': args.database,
        'user': args.user,
        'password': args.password,
        'port': args.port
    }

    # 创建下载器并运行
    downloader = WubiDataDownloader(db_config)

    # 检查数据库连接
    if not args.export_only:
        print("测试数据库连接...")
        conn = downloader.connect_database()
        if not conn:
            print("警告: 无法连接到数据库!")
            print("您可以选择:")
            print("  1. 检查数据库配置并重新运行")
            print("  2. 使用 --export-only 参数仅导出数据")
            response = input("\n是否继续仅导出数据? (y/n): ")
            if response.lower() != 'y':
                print("退出程序")
                return
            args.export_only = True

    success = downloader.run(export_only=args.export_only)

    if success:
        print("\n下一步操作建议:")
        print("  1. 检查生成的文件: wubi_data_insert.sql 和 wubi_data.csv")
        print("  2. 如果需要手动导入，使用命令: mysql -u root -p wubi < wubi_data_insert.sql")
        print("  3. 数据已准备好供应用程序使用")
    else:
        print("\n处理失败，请检查错误信息")
        sys.exit(1)


if __name__ == "__main__":
    main()