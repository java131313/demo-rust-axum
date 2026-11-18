#!/bin/bash
# 重新生成0001_schema.sql：逐表导出，避免grep按行过滤切断多行CREATE
set -e
cd "$(dirname "$0")/.."
DB=data/app.db
OUT=cloudflare-worker/migrations/0001_schema.sql
TABLES="articles bopomofo_characters bopomofo_keyboards custom_articles english_texts japanese_characters japanese_gojuon japanese_kanji japanese_keyboards japanese_texts key_radicals lessons traditional_chinese_texts user_progress users wubi_characters wubi_roots"
: > "$OUT"
for t in $TABLES; do
  sqlite3 "$DB" "SELECT sql || ';' FROM sqlite_master WHERE type='table' AND name='$t';" >> "$OUT"
  echo "" >> "$OUT"
done
# 校验语法
rm -f /tmp/test_d1.sql.db
sqlite3 /tmp/test_d1.sql.db < "$OUT"
sqlite3 /tmp/test_d1.sql.db < cloudflare-worker/migrations/0002_seed_basic.sql
echo "schema+seed 语法OK, tables: $(sqlite3 /tmp/test_d1.sql.db "SELECT COUNT(*) FROM sqlite_master WHERE type='table';")"
