"""把 liaozhai_articles.json 中的聊斋文章通过自定义文章接口导入线上部署。"""
import json
import time
import urllib.request
import urllib.error
from concurrent.futures import ThreadPoolExecutor, as_completed

API = "https://qmacumtdsubg.sealosbja.site/api/custom-articles"
DATA = "/Users/kangxiaoyu/program/demo-rust-axum/liaozhai_articles.json"
WORKERS = 3
RETRIES = 4

def post_article(article):
    payload = json.dumps({
        "title": article["title"],
        "content": article["content"],
        "difficulty": "custom",
    }).encode("utf-8")
    req = urllib.request.Request(
        API, data=payload,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    for attempt in range(1, RETRIES + 1):
        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                body = json.loads(resp.read().decode("utf-8"))
                return True, body.get("id"), article["title"], None
        except urllib.error.HTTPError as e:
            err = f"HTTP {e.code}: {e.read().decode('utf-8', 'ignore')[:100]}"
        except Exception as e:
            err = str(e)
        time.sleep(1.5 * attempt)
    return False, None, article["title"], err

def main():
    with open(DATA) as f:
        articles = json.load(f)
    print(f"共 {len(articles)} 篇，开始导入 -> {API}")
    ok, failed = 0, []
    with ThreadPoolExecutor(max_workers=WORKERS) as ex:
        futures = [ex.submit(post_article, a) for a in articles]
        for i, fut in enumerate(as_completed(futures), 1):
            success, aid, title, err = fut.result()
            if success:
                ok += 1
            else:
                failed.append((title, err))
            if i % 50 == 0 or i == len(articles):
                print(f"进度 {i}/{len(articles)} 成功 {ok} 失败 {len(failed)}")
    print(f"\n完成: 成功 {ok} / {len(articles)}")
    for title, err in failed:
        print(f"  失败: {title} -> {err}")

if __name__ == "__main__":
    main()