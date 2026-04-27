#!/usr/bin/env python3
"""爬取聊斋志异文章并插入 PostgreSQL 数据库"""

import getpass
import json
import os
import re
import time
import requests
import psycopg2
from bs4 import BeautifulSoup
from urllib.parse import urljoin

BASE_URL = "https://liaozhai.5000yan.com/"
INDEX_URL = "https://liaozhai.5000yan.com/"
CACHE_FILE = "liaozhai_articles.json"

DB_CONFIG = {
    "host": "127.0.0.1",
    "port": 5432,
    "user": "root",
    "password": "root",
    "dbname": "wubi",
}

HEADERS = {
    "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
    "Accept-Language": "zh-CN,zh;q=0.9,en;q=0.8",
}


def get_article_links():
    """从首页获取所有文章链接"""
    print(f"正在获取文章列表: {INDEX_URL}")
    resp = requests.get(INDEX_URL, headers=HEADERS, timeout=30)
    resp.encoding = "utf-8"
    soup = BeautifulSoup(resp.text, "lxml")

    links = []
    seen = set()
    for a in soup.find_all("a", href=re.compile(r"/\d+\.html$")):
        href = a.get("href")
        title = a.get_text(strip=True)
        if not href or not title:
            continue
        url = urljoin(BASE_URL, href)
        if url in seen:
            continue
        seen.add(url)
        links.append({"title": title, "url": url})

    print(f"共找到 {len(links)} 篇文章")
    return links


def fetch_article(url):
    """获取单篇文章的标题和内容"""
    resp = requests.get(url, headers=HEADERS, timeout=30)
    resp.encoding = "utf-8"
    soup = BeautifulSoup(resp.text, "lxml")

    # 标题
    title_tag = soup.select_one("h5.py-3.lh-base.text-center")
    if not title_tag:
        title_tag = soup.find("h5")
    title = title_tag.get_text(strip=True) if title_tag else ""

    # 内容区域
    grap = soup.select_one(".grap")
    if not grap:
        return title, ""

    paragraphs = []
    current_section = None

    for div in grap.find_all("div", recursive=False):
        text = div.get_text(strip=True)
        if not text:
            continue

        strong = div.find("strong")
        if strong:
            section_name = strong.get_text(strip=True)
            if section_name in ("【原文】", "【翻译】", "【点评】"):
                current_section = section_name
                continue

        paragraphs.append(text)

    content = "\n\n".join(paragraphs)
    return title, content


def save_cache(articles):
    """保存文章到本地缓存"""
    with open(CACHE_FILE, "w", encoding="utf-8") as f:
        json.dump(articles, f, ensure_ascii=False, indent=2)
    print(f"缓存已保存到 {CACHE_FILE}")


def load_cache():
    """从本地缓存加载文章"""
    if not os.path.exists(CACHE_FILE):
        return None
    with open(CACHE_FILE, "r", encoding="utf-8") as f:
        articles = json.load(f)
    print(f"从缓存加载了 {len(articles)} 篇文章")
    return articles


def insert_articles(articles):
    """插入文章到数据库"""
    conn = psycopg2.connect(**DB_CONFIG)
    cur = conn.cursor()

    inserted = 0
    skipped = 0
    for art in articles:
        title = art["title"]
        content = art["content"]
        if not title or not content:
            skipped += 1
            continue

        cur.execute(
            "INSERT INTO articles (title, content, difficulty) VALUES (%s, %s, %s) ON CONFLICT DO NOTHING",
            (title, content, "medium"),
        )
        inserted += 1

    conn.commit()
    cur.close()
    conn.close()
    return inserted, skipped


def main():
    # 尝试从缓存加载
    articles = load_cache()

    if articles is None:
        # 1. 获取文章链接
        links = get_article_links()
        if not links:
            print("未找到任何文章链接")
            return

        # 2. 爬取每篇文章
        articles = []
        total = len(links)
        for i, item in enumerate(links, 1):
            try:
                title, content = fetch_article(item["url"])
                if title and content:
                    articles.append({"title": title, "content": content})
                    print(f"[{i}/{total}] 成功: {title} ({len(content)} 字)")
                else:
                    print(f"[{i}/{total}] 跳过: {item['url']} (无内容)")
            except Exception as e:
                print(f"[{i}/{total}] 失败: {item['url']} - {e}")

            # 礼貌延迟
            time.sleep(0.5)

        print(f"\n成功爬取 {len(articles)} 篇文章")
        save_cache(articles)

    # 3. 插入数据库
    if articles:
        inserted, skipped = insert_articles(articles)
        print(f"数据库插入完成: {inserted} 条成功, {skipped} 条跳过")


if __name__ == "__main__":
    main()
