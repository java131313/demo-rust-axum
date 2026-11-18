#!/bin/bash
# 生成D1迁移文件：schema + 基础数据 + 聊斋 + 五笔字库
set -e
cd "$(dirname "$0")/.."
DB=data/app.db
M=cloudflare-worker/migrations

dump_table() {
  sqlite3 "$DB" ".mode insert $1" "SELECT * FROM $1;" | sed 's/^INSERT INTO/INSERT OR IGNORE INTO/'
}

{
  for t in lessons articles english_texts japanese_keyboards japanese_characters japanese_texts key_radicals wubi_roots bopomofo_keyboards bopomofo_characters traditional_chinese_texts japanese_gojuon japanese_kanji; do
    dump_table "$t"
  done
} > "$M/0002_seed_basic.sql"

dump_table custom_articles > "$M/0003_seed_articles.sql"
dump_table wubi_characters > "$M/0004_seed_wubi.sql"

for f in "$M"/*.sql; do
  echo "$f: $(wc -c < "$f") bytes"
done
