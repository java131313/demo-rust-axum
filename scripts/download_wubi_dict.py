#!/usr/bin/env python3
"""Download complete wubi dictionary from GitHub and save as JSON."""

import json
import urllib.request
import os

def download_wubi_dict():
    """Download wubi dictionary from GitHub vvk/wubi project."""
    
    # Try different sources
    urls = [
        "https://raw.githubusercontent.com/vvk/wubi/main/data/wubi86.json",
        "https://raw.githubusercontent.com/CNMan/UnicodeCJK-WuBi/master/wubi.txt",
    ]
    
    for url in urls:
        try:
            print(f"Downloading from: {url}")
            req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
            with urllib.request.urlopen(req, timeout=30) as response:
                data = json.loads(response.read().decode('utf-8'))
            
            print(f"Downloaded successfully. Type: {type(data)}")
            
            # Process the data
            entries = []
            seen = set()
            
            if isinstance(data, list):
                for item in data:
                    if isinstance(item, dict):
                        char = item.get('char', item.get('character', item.get('hanzi')))
                        code = item.get('wubi86', item.get('code', item.get('wubi')))
                        if char and len(char) == 1 and code and char not in seen:
                            seen.add(char)
                            entries.append({"character": char, "code": code.upper()})
            elif isinstance(data, dict):
                for char, info in data.items():
                    if len(char) == 1 and char not in seen:
                        code = info if isinstance(info, str) else info.get('wubi86', info.get('code'))
                        if code:
                            seen.add(char)
                            entries.append({"character": char, "code": code.upper()})
            
            print(f"Found {len(entries)} unique entries")
            
            # Save to JSON
            output_path = os.path.join(os.path.dirname(__file__), '..', 'data', 'wubi_dict.json')
            os.makedirs(os.path.dirname(output_path), exist_ok=True)
            
            with open(output_path, 'w', encoding='utf-8') as f:
                json.dump(entries, f, ensure_ascii=False, indent=2)
            
            print(f"Saved to: {output_path}")
            print(f"Total entries: {len(entries)}")
            return True
            
        except Exception as e:
            print(f"Failed to download from {url}: {e}")
            continue
    
    return False

if __name__ == "__main__":
    print("Downloading complete wubi dictionary from GitHub...")
    success = download_wubi_dict()
    
    if not success:
        print("\nError: Failed to download dictionary data")
        print("Please check internet connection or try again later")
        exit(1)
    
    print("\nDone! Dictionary data downloaded successfully")
