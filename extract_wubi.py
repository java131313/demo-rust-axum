#!/usr/bin/env python3
import re

with open('import_wubi_data.py', 'r', encoding='utf-8') as f:
    content = f.read()

# 使用正则表达式查找 wubi_data 列表
pattern = r'wubi_data = \[(.*?)\]'
match = re.search(pattern, content, re.DOTALL)
if not match:
    print("未找到 wubi_data 列表")
    exit(1)

data_text = match.group(1)
# 提取所有元组
tuple_pattern = r'\("([^"]+)", "([^"]+)"\)'
tuples = re.findall(tuple_pattern, data_text)

print(f"找到 {len(tuples)} 个元组")

# 去重
unique = {}
for char, code in tuples:
    if char not in unique:
        unique[char] = code

print(f"去重后: {len(unique)} 个字符")

# 输出为 Rust 数组格式
print("\nRust 数组:")
print("let wubi_data = &[")
for i, (char, code) in enumerate(unique.items()):
    if i % 10 == 0 and i > 0:
        print()
    print(f'    ("{char}", "{code}"),')
print("];")

# 也可以输出为 SQL 插入语句
print("\nSQL 插入语句 (前10个):")
for char, code in list(unique.items())[:10]:
    print(f'INSERT INTO wubi_characters (character_val, wubi_code) VALUES ("{char}", "{code}");')