#!/usr/bin/env python3
"""生成D1迁移：schema逐表导出 + seed数据(标准SQL转义,不用unistr)"""
import sqlite3, os, sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DB = os.path.join(ROOT, "data", "app.db")
M = os.path.join(ROOT, "cloudflare-worker", "migrations")

TABLES_BASIC = ["lessons", "articles", "english_texts", "japanese_keyboards",
                "japanese_characters", "japanese_texts", "key_radicals", "wubi_roots",
                "bopomofo_keyboards", "bopomofo_characters", "traditional_chinese_texts",
                "japanese_gojuon", "japanese_kanji"]
SCHEMA_TABLES = TABLES_BASIC + ["custom_articles", "user_progress", "users",
                                "wubi_characters"]

def q(v):
    if v is None:
        return "NULL"
    if isinstance(v, (int, float)):
        return str(v)
    return "'" + str(v).replace("'", "''") + "'"

def dump_table(cur, table, out):
    cols = [r[1] for r in cur.execute(f"PRAGMA table_info('{table}')")]
    collist = ", ".join(f'"{c}"' for c in cols)
    for row in cur.execute(f"SELECT * FROM {table}"):
        vals = ", ".join(q(v) for v in row)
        out.write(f"INSERT OR IGNORE INTO {table} ({collist}) VALUES({vals});\n")

def main():
    con = sqlite3.connect(DB)
    cur = con.cursor()
    os.makedirs(M, exist_ok=True)

    with open(os.path.join(M, "0001_schema.sql"), "w") as f:
        for t in SCHEMA_TABLES:
            row = cur.execute(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name=?", (t,)
            ).fetchone()
            if row:
                f.write(row[0].rstrip(";") + ";\n\n")

    with open(os.path.join(M, "0002_seed_basic.sql"), "w") as f:
        for t in TABLES_BASIC:
            dump_table(cur, t, f)

    with open(os.path.join(M, "0003_seed_articles.sql"), "w") as f:
        dump_table(cur, "custom_articles", f)

    with open(os.path.join(M, "0004_seed_wubi.sql"), "w") as f:
        dump_table(cur, "wubi_characters", f)

    con.close()
    for fn in sorted(os.listdir(M)):
        print(fn, os.path.getsize(os.path.join(M, fn)))

if __name__ == "__main__":
    main()
