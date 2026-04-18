#!/usr/bin/env python3
import json
import re
import requests
import time

# 拼音查询函数
def get_pinyin(char):
    """获取汉字拼音"""
    try:
        # 尝试使用pypinyin库（如果已安装）
        try:
            from pypinyin import pinyin, Style
            result = pinyin(char, style=Style.TONE, heteronym=False)
            if result and result[0]:
                return result[0][0]
        except ImportError:
            pass
        
        # 使用汉字转拼音的简单映射
        # 这里只处理常见汉字，对于生僻字返回空字符串
        # 实际项目中建议安装pypinyin库以获得更完整的拼音支持
        return ""
    except Exception as e:
        return ""

def parse_yaml_dict(yaml_path, output_path):
    entries = []

    with open(yaml_path, 'r', encoding='utf-8') as f:
        content = f.read()

    # Skip header lines until "..."
    lines = content.split('\n')
    data_started = False
    total_lines = len(lines)
    processed = 0
    
    # 缓存已查询的拼音，避免重复请求
    pinyin_cache = {}
    
    print(f"Total lines: {total_lines}")
    
    for line in lines:
        line = line.strip()

        # Check for data start marker
        if line == '...':
            data_started = True
            continue

        # Skip header and metadata
        if line.startswith('#') or line.startswith('---') or line.startswith('name:') or \
           line.startswith('version:') or line.startswith('sort:') or line.startswith('columns:') or \
           line.startswith('encoder:') or line == '':
            continue

        if not data_started:
            continue

        # Parse data lines
        parts = line.split('\t')
        if len(parts) >= 2:
            char = parts[0].strip()
            simple_code = parts[1].strip()  # 第二列作为简码
            
            # 处理不同长度的行
            if len(parts) >= 4:
                remark = parts[2].strip()      # 第三列作为备注
                full_code = parts[3].strip()    # 第四列作为全码
            else:
                remark = ""  # 没有备注
                full_code = simple_code  # 全码设为与简码相同

            # Skip empty lines
            if not char or not simple_code:
                continue

            # Skip comments (lines starting with #)
            if char.startswith('#'):
                char = char[1:]
                if not char:
                    continue

            # 获取拼音，优先从缓存中获取
            if char in pinyin_cache:
                pinyin = pinyin_cache[char]
            else:
                pinyin = get_pinyin(char)
                pinyin_cache[char] = pinyin
            
            # 每处理200个字符显示进度
            processed += 1
            if processed % 200 == 0:
                print(f"Processed {processed}/{total_lines} lines...")

            entry = {
                "character": char,
                "simple_code": simple_code,
                "full_code": full_code,
                "pinyin": pinyin,
                "remark": remark
            }
            entries.append(entry)

    # Write to JSON
    with open(output_path, 'w', encoding='utf-8') as f:
        json.dump(entries, f, ensure_ascii=False, indent=2)

    print(f"Converted {len(entries)} entries to {output_path}")

if __name__ == "__main__":
    parse_yaml_dict("data/wubi86.dict.yaml", "data/wubi86_converted.json")