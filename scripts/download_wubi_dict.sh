#!/usr/bin/env bash
# Download complete Wubi dictionary data from GitHub and convert to JSON format

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
DATA_DIR="$PROJECT_DIR/data"
OUTPUT_FILE="$DATA_DIR/wubi_dict.json"

mkdir -p "$DATA_DIR"

echo "Downloading Wubi dictionary data from GitHub..."

# Try downloading from multiple sources
download_url() {
    local url=$1
    echo "Trying: $url"
    if curl -s -f --max-time 30 "$url" -o "$DATA_DIR/temp_wubi.txt"; then
        echo "Success: downloaded from $url"
        return 0
    else
        echo "Failed: $url"
        return 1
    fi
}

# Try vvk/wubi first
if download_url "https://raw.githubusercontent.com/vvk/wubi/main/data/wubi86.json"; then
    INPUT_FILE="$DATA_DIR/temp_wubi.txt"
elif download_url "https://raw.githubusercontent.com/CNMan/UnicodeCJK-WuBi/master/wubi.txt"; then
    INPUT_FILE="$DATA_DIR/temp_wubi.txt"
else
    echo "ERROR: All download sources failed"
    exit 1
fi

echo "Converting to JSON format..."

# Convert the downloaded data to JSON using Python
python3 << 'PYTHON_SCRIPT'
import json
import sys
import os

input_file = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), 'temp_wubi.txt')
output_file = os.path.join(os.path.dirname(__file__), '..', 'data', 'wubi_dict.json')

entries = []

try:
    # Try JSON first
    with open(input_file, 'r', encoding='utf-8') as f:
        try:
            data = json.load(f)
            if isinstance(data, list):
                for item in data:
                    if isinstance(item, dict):
                        char = item.get('char', item.get('character', item.get('hanzi')))
                        code = item.get('wubi86', item.get('code', item.get('wubi')))
                        if char and code:
                            entries.append({"character": char, "code": code})
            elif isinstance(data, dict):
                for char, info in data.items():
                    if len(char) == 1:
                        code = info if isinstance(info, str) else info.get('wubi86', info.get('code'))
                        if code:
                            entries.append({"character": char, "code": code})
        except json.JSONDecodeError:
            # Parse as text
            with open(input_file, 'r', encoding='utf-8') as f:
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    if '\t' in line:
                        parts = line.split('\t')
                        if len(parts) >= 2:
                            char, code = parts[0].strip(), parts[1].strip()
                            if len(char) == 1 and code:
                                entries.append({"character": char, "code": code})
                    elif ' ' in line:
                        parts = line.split(' ', 1)
                        if len(parts) >= 2:
                            char, code = parts[0].strip(), parts[1].strip()
                            if len(char) == 1 and code:
                                entries.append({"character": char, "code": code})
except Exception as e:
    print(f"Error processing file: {e}")
    sys.exit(1)

# Remove duplicates
seen = set()
unique_entries = []
for entry in entries:
    if entry["character"] not in seen:
        seen.add(entry["character"])
        unique_entries.append(entry)

# Save JSON
os.makedirs(os.path.dirname(output_file), exist_ok=True)
with open(output_file, 'w', encoding='utf-8') as f:
    json.dump(unique_entries, f, ensure_ascii=False, indent=2)

print(f"Generated JSON file with {len(unique_entries)} entries")
PYTHON_SCRIPT

# Clean up
rm -f "$DATA_DIR/temp_wubi.txt"

echo "Done! Dictionary data saved to $OUTPUT_FILE"
echo "Total entries: $(python3 -c "import json; print(len(json.load(open('$OUTPUT_FILE'))))")"
