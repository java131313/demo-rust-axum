#!/usr/bin/env python3
"""
将 wubi86.dict.yaml 转换为带有拼音的 PostgreSQL 数据
适配 PostgreSQL 表结构: simple_code, full_code, pinyin, remark
处理重复的汉字 - 只保留第一个出现的条目
简码可以是1-2字符，全码必须是2-4字符
"""

from pypinyin import pinyin, Style
import re

def get_pinyin(char):
    """获取汉字拼音"""
    try:
        result = pinyin(char, style=Style.TONE, heteronym=False)
        if result and result[0]:
            return result[0][0]
    except Exception as e:
        return ""

def is_valid_wubi_code(code, allow_single=False):
    """检查是否是有效的五笔编码"""
    if not code:
        return False
    code_upper = code.upper()
    if code_upper.isdigit():
        return False
    if code_upper == 'ZZPP':
        return False
    if not re.match(r'^[AGQWERTZUIOPJKLBNMYFXCDVHS]+$', code_upper):
        return False
    if not allow_single and len(code_upper) < 2:
        return False
    if len(code_upper) > 4:
        return False
    return True

def parse_yaml_to_pg_sql(yaml_path, output_sql_path):
    """解析YAML文件并生成PostgreSQL SQL脚本"""

    entries_dict = {}

    with open(yaml_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    data_started = False
    total_lines = len(lines)
    processed = 0
    skipped = 0
    skipped_code = 0

    print(f"Total lines: {total_lines}")

    for line_num, line in enumerate(lines, 1):
        line = line.strip()

        if line == '...':
            data_started = True
            continue

        if line.startswith('#') or line.startswith('---') or line.startswith('name:') or \
           line.startswith('version:') or line.startswith('sort:') or line.startswith('columns:') or \
           line.startswith('encoder:') or line == '':
            continue

        if not data_started:
            continue

        parts = line.split('\t')
        if len(parts) >= 2:
            char = parts[0].strip()
            simple_code = parts[1].strip()

            if not char or not simple_code:
                continue

            if char.startswith('#'):
                char = char[1:]
                if not char:
                    continue

            if len(char) != 1 or not re.match(r'[\u4e00-\u9fff]', char):
                skipped += 1
                continue

            if not is_valid_wubi_code(simple_code, allow_single=True):
                skipped_code += 1
                continue

            full_code = ""
            if len(parts) >= 4 and parts[3].strip():
                candidate_code = parts[3].strip()
                if is_valid_wubi_code(candidate_code):
                    full_code = candidate_code
            elif len(parts) >= 3 and parts[2].strip():
                candidate_code = parts[2].strip()
                if is_valid_wubi_code(candidate_code):
                    full_code = candidate_code

            if not full_code:
                full_code = simple_code

            if char not in entries_dict:
                pinyin_val = get_pinyin(char)
                entries_dict[char] = (char, simple_code, full_code, pinyin_val, "")
                processed += 1

            if processed % 1000 == 0:
                print(f"Processed {processed}/{total_lines} lines...")

    entries = list(entries_dict.values())

    print(f"\nTotal processed: {processed}, Skipped (not Chinese): {skipped}, Skipped (invalid code): {skipped_code}")

    sql_content = """-- PostgreSQL更新脚本
-- 从 wubi86.dict.yaml 导入数据到 wubi_characters 表

TRUNCATE TABLE wubi_characters;

INSERT INTO wubi_characters (character_val, simple_code, full_code, pinyin, remark) VALUES
"""

    values = []
    for char, simple_code, full_code, pinyin_val, remark in entries:
        char_escaped = char.replace("'", "''")
        pinyin_escaped = pinyin_val.replace("'", "''") if pinyin_val else "''"
        remark_escaped = remark.replace("'", "''") if remark else "''"
        values.append(f"('{char_escaped}', '{simple_code}', '{full_code}', '{pinyin_escaped}', '{remark_escaped}')")

    sql_content += ",\n".join(values)
    sql_content += "\nON CONFLICT (character_val) DO UPDATE SET\n    simple_code = EXCLUDED.simple_code,\n    full_code = EXCLUDED.full_code,\n    pinyin = EXCLUDED.pinyin,\n    remark = EXCLUDED.remark;\n\n"
    sql_content += f"SELECT '数据导入完成! 共导入 ' || COUNT(*) || ' 条记录' AS message FROM wubi_characters;\n"

    with open(output_sql_path, 'w', encoding='utf-8') as f:
        f.write(sql_content)

    print(f"\nGenerated SQL script: {output_sql_path}")
    print(f"Total entries (deduplicated): {len(entries)}")

    print("\nSample data:")
    for i in range(min(20, len(entries))):
        char, simple_code, full_code, pinyin_val, remark = entries[i]
        print(f"  {char}: 简码={simple_code} 全码={full_code} 拼音={pinyin_val}")

if __name__ == "__main__":
    yaml_path = "data/wubi86.dict.yaml"
    output_sql_path = "wubi86_pg.sql"
    parse_yaml_to_pg_sql(yaml_path, output_sql_path)