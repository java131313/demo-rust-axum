//! Redis缓存层：包装主数据库（SQLite），为读多写少的数据提供Redis缓存。
//! Redis不可用时自动降级为直接访问数据库，保证服务可用性。

use crate::config::{
    Article, BopomofoCharacter, BopomofoKeyboard, EnglishText, JapaneseCharacter, JapaneseGojuon,
    JapaneseKanji, JapaneseKeyboard, JapaneseText, KeyRadical, Lesson, TraditionalChineseText,
    User, WubiCharacter, WubiRoot,
};
use crate::db::Database;
use async_trait::async_trait;
use redis::AsyncCommands;
use std::sync::Arc;

/// 缓存键前缀
const KEY_PREFIX: &str = "wubi:cache";
/// 列表类数据缓存时间（秒）
const LIST_TTL: u64 = 300;
/// 自定义文章列表缓存时间（秒）：内容大且变更少，单独设置更长TTL
const CUSTOM_ARTICLES_TTL: u64 = 1800;
/// 五笔编码查询缓存时间（秒）
const WUBI_TTL: u64 = 600;
/// Redis操作超时（秒）：跨境网络抖动时快速失败，降级为直读数据库
const REDIS_OP_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(3);

/// 带Redis缓存的数据库包装器
pub struct CachedDatabase {
    inner: Arc<dyn Database>,
    redis: redis::aio::ConnectionManager,
}

impl CachedDatabase {
    pub fn new(inner: Arc<dyn Database>, redis: redis::aio::ConnectionManager) -> Self {
        Self { inner, redis }
    }

    /// 从缓存读取JSON数据，未命中则回源并写入缓存
    async fn cached<T>(&self, key: &str, ttl: u64, fetch: impl std::future::Future<Output = Result<T, String>>) -> Result<T, String>
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Clone,
    {
        let full_key = format!("{}:{}", KEY_PREFIX, key);
        let mut conn = self.redis.clone();

        // 先查缓存（带超时，网络抖动时快速降级）
        let cached_value = tokio::time::timeout(
            REDIS_OP_TIMEOUT,
            conn.get::<_, Option<String>>(&full_key),
        )
        .await;
        match cached_value {
            Ok(Ok(Some(json))) => match serde_json::from_str::<T>(&json) {
                Ok(value) => return Ok(value),
                Err(e) => {
                    tracing::warn!("cache deserialize failed for {}: {}", full_key, e);
                }
            },
            Ok(Ok(None)) => {}
            Ok(Err(e)) => {
                tracing::warn!("redis GET {} failed: {}, fallback to db", full_key, e);
            }
            Err(_) => {
                tracing::warn!("redis GET {} timeout, fallback to db", full_key);
            }
        }

        // 回源
        let value = fetch.await?;

        // 写入缓存（失败仅告警，不影响响应）
        if let Ok(json) = serde_json::to_string(&value) {
            let set_result = tokio::time::timeout(
                REDIS_OP_TIMEOUT,
                conn.set_ex::<_, _, ()>(&full_key, json, ttl),
            )
            .await;
            match set_result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => tracing::warn!("redis SET {} failed: {}", full_key, e),
                Err(_) => tracing::warn!("redis SET {} timeout", full_key),
            }
        }

        Ok(value)
    }

    /// 删除缓存键（写入后调用，忽略错误）
    async fn invalidate(&self, keys: &[&str]) {
        let full_keys: Vec<String> = keys
            .iter()
            .map(|k| format!("{}:{}", KEY_PREFIX, k))
            .collect();
        let mut conn = self.redis.clone();
        let _ = tokio::time::timeout(REDIS_OP_TIMEOUT, conn.del::<_, ()>(full_keys)).await;
    }

    /// 预热缓存：把前端加载的文章类数据写入Redis，避免首个请求穿透到SQLite
    pub async fn warm_cache(&self) {
        tracing::info!("Warming Redis cache for article lists...");
        let _ = self.get_articles().await;
        for lang in ["zh", "ja", "tw", "en"] {
            let _ = self.get_custom_articles(lang).await;
        }
        let _ = self.get_english_texts().await;
        let _ = self.get_japanese_texts().await;
        let _ = self.get_traditional_chinese_texts().await;
        let _ = self.get_lessons().await;
        let _ = self.get_japanese_gojuon().await;
        let _ = self.get_key_radicals().await;
        tracing::info!("Redis cache warmed");
    }
}

#[async_trait]
impl Database for CachedDatabase {
    async fn health_check(&self) -> Result<(), String> {
        self.inner.health_check().await?;
        let mut conn = self.redis.clone();
        let ping = tokio::time::timeout(REDIS_OP_TIMEOUT, redis::cmd("PING").query_async::<_, ()>(&mut conn))
            .await;
        match ping {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err("redis ping timeout".to_string()),
        }
    }

    async fn init_db(&self) -> Result<(), String> {
        self.inner.init_db().await
    }

    async fn get_wubi_code(&self, character: &str) -> Result<WubiCharacter, String> {
        self.cached(
            &format!("wubi:{}", character),
            WUBI_TTL,
            self.inner.get_wubi_code(character),
        )
        .await
    }

    async fn get_all_wubi_characters(&self) -> Result<Vec<WubiCharacter>, String> {
        // 数据量过大（13万+条），不走Redis缓存
        self.inner.get_all_wubi_characters().await
    }

    async fn get_lessons(&self) -> Result<Vec<Lesson>, String> {
        self.cached("lessons", LIST_TTL, self.inner.get_lessons())
            .await
    }

    async fn get_lesson_by_id(&self, id: i32) -> Result<Lesson, String> {
        self.cached(
            &format!("lesson:{}", id),
            LIST_TTL,
            self.inner.get_lesson_by_id(id),
        )
        .await
    }

    async fn create_lesson(
        &self,
        character: &str,
        code: &str,
        description: &str,
    ) -> Result<Lesson, String> {
        let lesson = self
            .inner
            .create_lesson(character, code, description)
            .await?;
        self.invalidate(&["lessons"]).await;
        Ok(lesson)
    }

    async fn get_articles(&self) -> Result<Vec<Article>, String> {
        self.cached("articles", LIST_TTL, self.inner.get_articles())
            .await
    }

    async fn get_article_by_id(&self, id: i32) -> Result<Article, String> {
        self.cached(
            &format!("article:{}", id),
            LIST_TTL,
            self.inner.get_article_by_id(id),
        )
        .await
    }

    async fn create_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let article = self
            .inner
            .create_article(title, content, difficulty)
            .await?;
        self.invalidate(&["articles"]).await;
        Ok(article)
    }

    async fn update_article(
        &self,
        id: i32,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let article = self
            .inner
            .update_article(id, title, content, difficulty)
            .await?;
        self.invalidate(&["articles", &format!("article:{}", id)])
            .await;
        Ok(article)
    }

    async fn delete_article(&self, id: i32) -> Result<(), String> {
        self.inner.delete_article(id).await?;
        self.invalidate(&["articles", &format!("article:{}", id)])
            .await;
        Ok(())
    }

    async fn get_custom_articles(&self, lang: &str) -> Result<Vec<Article>, String> {
        self.cached(
            &format!("custom-articles:{}", lang),
            CUSTOM_ARTICLES_TTL,
            self.inner.get_custom_articles(lang),
        )
        .await
    }

    async fn create_custom_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
        lang: &str,
    ) -> Result<Article, String> {
        let article = self
            .inner
            .create_custom_article(title, content, difficulty, lang)
            .await?;
        self.invalidate(&[&format!("custom-articles:{}", lang)])
            .await;
        Ok(article)
    }

    async fn delete_custom_article(&self, id: i32) -> Result<(), String> {
        self.inner.delete_custom_article(id).await?;
        // 各语言的列表缓存都失效（无法确定文章所属语言）
        self.invalidate(&["custom-articles:zh", "custom-articles:ja", "custom-articles:tw"])
            .await;
        Ok(())
    }

    async fn update_custom_article(
        &self,
        id: i32,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let article = self
            .inner
            .update_custom_article(id, title, content, difficulty)
            .await?;
        self.invalidate(&["custom-articles:zh", "custom-articles:ja", "custom-articles:tw"])
            .await;
        Ok(article)
    }

    async fn update_wubi_code(
        &self,
        character: &str,
        new_code: &str,
    ) -> Result<WubiCharacter, String> {
        let result = self.inner.update_wubi_code(character, new_code).await?;
        self.invalidate(&[&format!("wubi:{}", character)]).await;
        Ok(result)
    }

    async fn get_wubi_roots(&self) -> Result<Vec<WubiRoot>, String> {
        self.cached("wubi-roots", LIST_TTL, self.inner.get_wubi_roots())
            .await
    }

    async fn get_wubi_root_by_id(&self, id: i32) -> Result<WubiRoot, String> {
        self.cached(
            &format!("wubi-root:{}", id),
            LIST_TTL,
            self.inner.get_wubi_root_by_id(id),
        )
        .await
    }

    async fn create_wubi_root(
        &self,
        character: &str,
        code: &str,
        position: &str,
        description: &str,
    ) -> Result<WubiRoot, String> {
        let root = self
            .inner
            .create_wubi_root(character, code, position, description)
            .await?;
        self.invalidate(&["wubi-roots"]).await;
        Ok(root)
    }

    async fn search_wubi_root(&self, character: &str) -> Result<Option<WubiRoot>, String> {
        self.cached(
            &format!("wubi-root-search:{}", character),
            LIST_TTL,
            self.inner.search_wubi_root(character),
        )
        .await
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String> {
        self.inner.get_user_by_username(username).await
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
        self.inner.get_user_by_id(id).await
    }

    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, String> {
        self.inner.create_user(username, email, password_hash).await
    }

    async fn save_progress(
        &self,
        user_name: &str,
        lesson_id: i32,
        accuracy: f32,
        score: i32,
    ) -> Result<(), String> {
        self.inner
            .save_progress(user_name, lesson_id, accuracy, score)
            .await
    }

    async fn get_key_radicals(&self) -> Result<Vec<KeyRadical>, String> {
        self.cached("key-radicals", LIST_TTL, self.inner.get_key_radicals())
            .await
    }

    async fn get_key_radical_by_key(&self, key_char: &str) -> Result<Option<KeyRadical>, String> {
        self.cached(
            &format!("key-radical:{}", key_char),
            LIST_TTL,
            self.inner.get_key_radical_by_key(key_char),
        )
        .await
    }

    async fn get_english_texts(&self) -> Result<Vec<EnglishText>, String> {
        self.cached("english-texts", LIST_TTL, self.inner.get_english_texts())
            .await
    }

    async fn get_japanese_texts(&self) -> Result<Vec<JapaneseText>, String> {
        self.cached("japanese-texts", LIST_TTL, self.inner.get_japanese_texts())
            .await
    }

    async fn get_japanese_keyboards(&self) -> Result<Vec<JapaneseKeyboard>, String> {
        self.cached(
            "japanese-keyboards",
            LIST_TTL,
            self.inner.get_japanese_keyboards(),
        )
        .await
    }

    async fn get_japanese_characters(&self) -> Result<Vec<JapaneseCharacter>, String> {
        self.cached(
            "japanese-characters",
            LIST_TTL,
            self.inner.get_japanese_characters(),
        )
        .await
    }

    async fn get_traditional_chinese_texts(&self) -> Result<Vec<TraditionalChineseText>, String> {
        self.cached(
            "traditional-chinese-texts",
            LIST_TTL,
            self.inner.get_traditional_chinese_texts(),
        )
        .await
    }

    async fn get_bopomofo_keyboards(&self) -> Result<Vec<BopomofoKeyboard>, String> {
        self.cached(
            "bopomofo-keyboards",
            LIST_TTL,
            self.inner.get_bopomofo_keyboards(),
        )
        .await
    }

    async fn get_bopomofo_characters(&self) -> Result<Vec<BopomofoCharacter>, String> {
        self.cached(
            "bopomofo-characters",
            LIST_TTL,
            self.inner.get_bopomofo_characters(),
        )
        .await
    }

    async fn get_japanese_gojuon(&self) -> Result<Vec<JapaneseGojuon>, String> {
        self.cached("japanese-gojuon", LIST_TTL, self.inner.get_japanese_gojuon())
            .await
    }

    async fn get_japanese_kanji(&self) -> Result<Vec<JapaneseKanji>, String> {
        self.cached("japanese-kanji", LIST_TTL, self.inner.get_japanese_kanji())
            .await
    }

    async fn get_japanese_kanji_by_char(
        &self,
        kanji: &str,
    ) -> Result<Option<JapaneseKanji>, String> {
        self.cached(
            &format!("japanese-kanji:{}", kanji),
            LIST_TTL,
            self.inner.get_japanese_kanji_by_char(kanji),
        )
        .await
    }
}
