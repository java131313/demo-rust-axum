use crate::config::{
    Article, BopomofoCharacter, BopomofoKeyboard, EnglishText, JapaneseCharacter, JapaneseGojuon,
    JapaneseKanji, JapaneseKeyboard, JapaneseText, KeyRadical, Lesson, TraditionalChineseText,
    User, WubiCharacter, WubiRoot,
};
use async_trait::async_trait;
use futures::stream::TryStreamExt;
use mongodb::bson::{Document, doc};
use mongodb::options::{FindOneAndUpdateOptions, FindOptions, ReturnDocument};
use mongodb::{Client, Database as MongoDb};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool, Postgres};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct WubiDictEntry {
    character: String,
    simple_code: String,
    full_code: String,
    pinyin: String,
    remark: String,
}

/// 数据库池的枚举类型
#[derive(Clone)]
pub enum DbPool {
    MySql(Pool<MySql>),
    Postgres(Pool<Postgres>),
}

/// 数据库操作trait
#[async_trait]
pub trait Database: Send + Sync {
    /// 初始化数据库表结构
    async fn init_db(&self) -> Result<(), String>;

    /// 获取五笔编码
    async fn get_wubi_code(&self, character: &str) -> Result<WubiCharacter, String>;

    /// 获取所有五笔字符
    async fn get_all_wubi_characters(&self) -> Result<Vec<WubiCharacter>, String>;

    /// 获取课程列表
    async fn get_lessons(&self) -> Result<Vec<Lesson>, String>;

    /// 根据ID获取课程
    async fn get_lesson_by_id(&self, id: i32) -> Result<Lesson, String>;

    /// 创建课程
    async fn create_lesson(
        &self,
        character: &str,
        code: &str,
        description: &str,
    ) -> Result<Lesson, String>;

    /// 获取文章列表
    async fn get_articles(&self) -> Result<Vec<Article>, String>;

    /// 根据ID获取文章
    async fn get_article_by_id(&self, id: i32) -> Result<Article, String>;

    /// 创建文章
    async fn create_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String>;

    /// 更新文章
    async fn update_article(
        &self,
        id: i32,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String>;

    /// 删除文章
    async fn delete_article(&self, id: i32) -> Result<(), String>;

    /// 获取所有自定义文章
    async fn get_custom_articles(&self) -> Result<Vec<Article>, String>;

    /// 创建自定义文章
    async fn create_custom_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String>;

    /// 删除自定义文章
    async fn delete_custom_article(&self, id: i32) -> Result<(), String>;

    /// 更新五笔编码
    async fn update_wubi_code(
        &self,
        character: &str,
        new_code: &str,
    ) -> Result<WubiCharacter, String>;

    /// 获取字根列表
    async fn get_wubi_roots(&self) -> Result<Vec<WubiRoot>, String>;

    /// 根据ID获取字根
    async fn get_wubi_root_by_id(&self, id: i32) -> Result<WubiRoot, String>;

    /// 创建字根
    async fn create_wubi_root(
        &self,
        character: &str,
        code: &str,
        position: &str,
        description: &str,
    ) -> Result<WubiRoot, String>;

    /// 根据字符搜索字根
    async fn search_wubi_root(&self, character: &str) -> Result<Option<WubiRoot>, String>;

    /// 根据用户名获取用户
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String>;

    /// 根据ID获取用户
    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String>;

    /// 创建用户
    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, String>;

    /// 保存用户进度
    async fn save_progress(
        &self,
        user_name: &str,
        lesson_id: i32,
        accuracy: f32,
        score: i32,
    ) -> Result<(), String>;

    /// 获取所有键位字根
    async fn get_key_radicals(&self) -> Result<Vec<KeyRadical>, String>;

    /// 根据键位获取字根
    async fn get_key_radical_by_key(&self, key_char: &str) -> Result<Option<KeyRadical>, String>;

    /// 获取所有英语练习文章
    async fn get_english_texts(&self) -> Result<Vec<EnglishText>, String>;

    /// 获取所有日语练习文章
    async fn get_japanese_texts(&self) -> Result<Vec<JapaneseText>, String>;

    /// 获取所有日语键盘布局
    async fn get_japanese_keyboards(&self) -> Result<Vec<JapaneseKeyboard>, String>;

    /// 获取所有日语字符
    async fn get_japanese_characters(&self) -> Result<Vec<JapaneseCharacter>, String>;

    /// 获取所有繁体中文练习文章
    async fn get_traditional_chinese_texts(&self) -> Result<Vec<TraditionalChineseText>, String>;

    /// 获取所有注音键盘布局
    async fn get_bopomofo_keyboards(&self) -> Result<Vec<BopomofoKeyboard>, String>;

    /// 获取所有注音字符
    async fn get_bopomofo_characters(&self) -> Result<Vec<BopomofoCharacter>, String>;

    /// 获取所有日语五十音
    async fn get_japanese_gojuon(&self) -> Result<Vec<JapaneseGojuon>, String>;

    /// 获取所有日语汉字
    async fn get_japanese_kanji(&self) -> Result<Vec<JapaneseKanji>, String>;

    /// 根据汉字查询日语发音
    async fn get_japanese_kanji_by_char(
        &self,
        kanji: &str,
    ) -> Result<Option<JapaneseKanji>, String>;
}

/// MySQL数据库实现
pub struct MySqlDatabase {
    pool: Pool<MySql>,
}

impl MySqlDatabase {
    pub fn new(pool: Pool<MySql>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Database for MySqlDatabase {
    async fn init_db(&self) -> Result<(), String> {
        // 创建表结构
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INT AUTO_INCREMENT PRIMARY KEY,
                username VARCHAR(64) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE,
                password_hash VARCHAR(255) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lessons (
                id INT AUTO_INCREMENT PRIMARY KEY,
                character_val VARCHAR(32) NOT NULL,
                code VARCHAR(32) NOT NULL,
                description TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS articles (
                id INT AUTO_INCREMENT PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium'
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS custom_articles (
                id INT AUTO_INCREMENT PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'custom'
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wubi_characters (
                id INT AUTO_INCREMENT PRIMARY KEY,
                character_val VARCHAR(32) NOT NULL UNIQUE,
                simple_code VARCHAR(8) NOT NULL DEFAULT '',
                full_code VARCHAR(8) NOT NULL DEFAULT '',
                pinyin VARCHAR(32) NOT NULL DEFAULT '',
                remark VARCHAR(128) NOT NULL DEFAULT '',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wubi_roots (
                id INT AUTO_INCREMENT PRIMARY KEY,
                character_val VARCHAR(32) NOT NULL,
                code VARCHAR(32) NOT NULL,
                position VARCHAR(64) NOT NULL,
                description TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_progress (
                id SERIAL PRIMARY KEY,
                user_name VARCHAR(64) NOT NULL,
                lesson_id INT NOT NULL,
                accuracy FLOAT NOT NULL,
                score INT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS key_radicals (
                id SERIAL PRIMARY KEY,
                key_char VARCHAR(4) NOT NULL UNIQUE,
                radicals TEXT NOT NULL,
                formula TEXT,
                description TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 创建日语相关表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS japanese_keyboards (
                id INT AUTO_INCREMENT PRIMARY KEY,
                layout_name VARCHAR(100) NOT NULL UNIQUE,
                description TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS japanese_characters (
                id INT AUTO_INCREMENT PRIMARY KEY,
                `character` VARCHAR(10) NOT NULL,
                romanji VARCHAR(50) NOT NULL,
                keyboard_key VARCHAR(10) NOT NULL,
                layout_id INT NOT NULL,
                FOREIGN KEY (layout_id) REFERENCES japanese_keyboards(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS japanese_texts (
                id INT AUTO_INCREMENT PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium',
                type VARCHAR(20) DEFAULT 'mixed'
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 插入示例数据
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM lessons")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if count == 0 {
            let lessons = [
                ("人", "W", "单人旁，常用字根"),
                ("口", "K", "口字旁，常用字根"),
                ("日", "J", "日字旁，常用字根"),
            ];

            for (char, code, desc) in lessons {
                sqlx::query(
                    "INSERT INTO lessons (character_val, code, description) VALUES (?, ?, ?)",
                )
                .bind(char)
                .bind(code)
                .bind(desc)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化键位字根数据
        let key_radical_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM key_radicals")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if key_radical_count == 0 {
            let key_radicals = [
                (
                    "g",
                    "王、一、五、戋",
                    "王旁青头戋（兼）五一",
                    "G区横区第一键，包含横笔和戈字根",
                ),
                (
                    "f",
                    "土、士、二、干、十、寸、雨",
                    "土士二干十寸雨",
                    "F区横区第二键，包含土字根",
                ),
                (
                    "d",
                    "大、犬、三、古、石、厂",
                    "大犬三（古）石厂",
                    "D区横区第三键，包含大字根",
                ),
                ("s", "木、丁、西", "木丁西", "S区横区第四键，包含木字根"),
                (
                    "a",
                    "工、戈、艹、七、廿",
                    "工戈草头右框七",
                    "A区横区第五键，包含工字根",
                ),
                (
                    "h",
                    "目、止、卜、虍、上",
                    "目具上止卜虎皮",
                    "H区竖区第一键，包含目字根",
                ),
                (
                    "j",
                    "日、早、虫、刂、竖",
                    "日早两竖与虫依",
                    "J区竖区第二键，包含日字根",
                ),
                ("k", "口、川", "口与川，字根稀", "K区竖区第三键，包含口字根"),
                (
                    "l",
                    "田、甲、四、车、囗",
                    "田甲方框四车里",
                    "L区竖区第四键，包含田字根",
                ),
                (
                    "m",
                    "山、由、贝、几",
                    "山由贝，下框几",
                    "M区竖区第五键，包含山字根",
                ),
                (
                    "t",
                    "禾、竹、丿、彳、攵",
                    "禾竹一撇双人立",
                    "T区撇区第一键，包含禾字根",
                ),
                (
                    "r",
                    "白、手、斤、牛",
                    "白手看头三二斤",
                    "R区撇区第二键，包含白字根",
                ),
                (
                    "e",
                    "舟、用、月、豕、衣",
                    "舟用乃月豕（家）衣",
                    "E区撇区第三键，包含月字根",
                ),
                (
                    "w",
                    "人、八、亻",
                    "人八登头单人几",
                    "W区撇区第四键，包含人字根",
                ),
                (
                    "q",
                    "金、饣、勹、儿、夕",
                    "金勺缺点无尾鱼，犬旁留叉",
                    "Q区撇区第五键，包含金字根",
                ),
                (
                    "y",
                    "言、文、方、广、丶",
                    "言文方广在四一，高头一捺谁人去",
                    "Y区捺区第一键，包含言字根",
                ),
                (
                    "u",
                    "立、辛、六、门、疒",
                    "立辛两点六门疒（病）",
                    "U区捺区第二键，包含立字根",
                ),
                (
                    "i",
                    "氵（三点水）、小",
                    "水旁兴头小倒立",
                    "I区捺区第三键，包含水字根",
                ),
                (
                    "o",
                    "火、米、灬",
                    "火业头，四点米",
                    "O区捺区第四键，包含火字根",
                ),
                (
                    "p",
                    "之、宀（宝盖）、冖、礻、衤",
                    "之字军盖建道底，摘礻衤",
                    "P区捺区第五键，包含之字根",
                ),
                (
                    "n",
                    "已、己、巳、尸、心、羽",
                    "已半巳满不出己，左框折尸心和羽",
                    "N区折区第一键，包含已字根",
                ),
                (
                    "b",
                    "子、耳、了、也、卩",
                    "子耳了也框向上",
                    "B区折区第二键，包含子字根",
                ),
                (
                    "v",
                    "女、刀、九、臼",
                    "女刀九臼山朝西",
                    "V区折区第三键，包含女字根",
                ),
                (
                    "c",
                    "又、巴、马、厶",
                    "又巴马，丢矢矣",
                    "C区折区第四键，包含又字根",
                ),
                (
                    "x",
                    "幺、母、弓、匕",
                    "慈母无心弓和匕，幼无力",
                    "X区折区第五键，包含丝字根",
                ),
            ];

            for (key, radicals, formula, desc) in key_radicals {
                sqlx::query(
                    "INSERT INTO key_radicals (key_char, radicals, formula, description) VALUES (?, ?, ?, ?)"
                )
                .bind(key)
                .bind(radicals)
                .bind(formula)
                .bind(desc)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化日语键盘布局数据
        let japanese_keyboard_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM japanese_keyboards")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if japanese_keyboard_count == 0 {
            let keyboard_layouts = [(
                "QWERTY Japanese",
                "标准QWERTY键盘的日语布局，使用罗马字输入",
            )];

            for (layout_name, description) in keyboard_layouts {
                sqlx::query(
                    "INSERT INTO japanese_keyboards (layout_name, description) VALUES (?, ?)",
                )
                .bind(layout_name)
                .bind(description)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化日语字符数据
        let japanese_character_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM japanese_characters")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if japanese_character_count == 0 {
            let japanese_chars = [
                ("あ", "a", "a", 1),
                ("い", "i", "i", 1),
                ("う", "u", "u", 1),
                ("え", "e", "e", 1),
                ("お", "o", "o", 1),
                ("か", "ka", "k", 1),
                ("き", "ki", "k", 1),
                ("く", "ku", "k", 1),
                ("け", "ke", "k", 1),
                ("こ", "ko", "k", 1),
                ("さ", "sa", "s", 1),
                ("し", "shi", "s", 1),
                ("す", "su", "s", 1),
                ("せ", "se", "s", 1),
                ("そ", "so", "s", 1),
            ];

            for (character, romanji, keyboard_key, layout_id) in japanese_chars {
                sqlx::query(
                    "INSERT INTO japanese_characters (`character`, romanji, keyboard_key, layout_id) VALUES (?, ?, ?, ?)"
                )
                .bind(character)
                .bind(romanji)
                .bind(keyboard_key)
                .bind(layout_id)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化日语文章数据
        let japanese_text_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM japanese_texts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if japanese_text_count == 0 {
            let japanese_texts = [
                (
                    "平假名基础",
                    "あいうえおかきくけこさしすせそ",
                    "easy",
                    "hiragana",
                ),
                (
                    "片假名基础",
                    "アイウエオカキクケコサシスセソ",
                    "easy",
                    "katakana",
                ),
                ("汉字基础", "一二三四五六七八九十", "easy", "kanji"),
                (
                    "混合练习",
                    "こんにちは、元気ですか？私は元気です。",
                    "easy",
                    "mixed",
                ),
                (
                    "平假名进阶",
                    "いろはにほへとちりぬるをわがよたれぞつねならむ",
                    "medium",
                    "hiragana",
                ),
                (
                    "片假名进阶",
                    "イロハニホヘトチリヌルヲワガヨタレゾツネナラム",
                    "medium",
                    "katakana",
                ),
                (
                    "汉字进阶",
                    "日本語の勉強は楽しいです。毎日勉強しています。",
                    "medium",
                    "kanji",
                ),
                (
                    "混合进阶",
                    "東京は日本の首都です。人口が多くてにぎやかです。",
                    "medium",
                    "mixed",
                ),
                (
                    "高级平假名",
                    "むかしむかしあるところにおじいさんとおばあさんがいました",
                    "hard",
                    "hiragana",
                ),
                (
                    "高级片假名",
                    "ムカシムカシアルトコロニオジイサントオバアサンガイマシタ",
                    "hard",
                    "katakana",
                ),
                (
                    "高级汉字",
                    "日本国は、東アジアの島国であり、首都は東京です。",
                    "hard",
                    "kanji",
                ),
                (
                    "高级混合",
                    "今日はいい天気ですね。公園に行きませんか？",
                    "hard",
                    "mixed",
                ),
            ];

            for (title, content, difficulty, r#type) in japanese_texts {
                sqlx::query(
                    "INSERT INTO japanese_texts (title, content, difficulty, type) VALUES (?, ?, ?, ?)"
                )
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .bind(r#type)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 创建繁体中文注音相关表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bopomofo_keyboards (
                id INT AUTO_INCREMENT PRIMARY KEY,
                layout_name VARCHAR(100) NOT NULL UNIQUE,
                description TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bopomofo_characters (
                id INT AUTO_INCREMENT PRIMARY KEY,
                `character` VARCHAR(10) NOT NULL,
                bopomofo VARCHAR(50) NOT NULL,
                keyboard_key VARCHAR(10) NOT NULL,
                layout_id INT NOT NULL,
                FOREIGN KEY (layout_id) REFERENCES bopomofo_keyboards(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS traditional_chinese_texts (
                id INT AUTO_INCREMENT PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium'
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 初始化注音键盘布局数据
        let bopomofo_keyboard_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM bopomofo_keyboards")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if bopomofo_keyboard_count == 0 {
            let keyboard_layouts = [("Bopomofo Keyboard", "标准注音键盘布局，使用注音符号输入")];

            for (layout_name, description) in keyboard_layouts {
                sqlx::query(
                    "INSERT INTO bopomofo_keyboards (layout_name, description) VALUES (?, ?)",
                )
                .bind(layout_name)
                .bind(description)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化注音字符数据
        let bopomofo_character_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM bopomofo_characters")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if bopomofo_character_count == 0 {
            let bopomofo_chars = [
                ("ㄅ", "b", "b", 1),
                ("ㄆ", "p", "p", 1),
                ("ㄇ", "m", "m", 1),
                ("ㄈ", "f", "f", 1),
                ("ㄉ", "d", "d", 1),
                ("ㄊ", "t", "t", 1),
                ("ㄋ", "n", "n", 1),
                ("ㄌ", "l", "l", 1),
                ("ㄍ", "g", "g", 1),
                ("ㄎ", "k", "k", 1),
                ("ㄏ", "h", "h", 1),
                ("ㄐ", "j", "j", 1),
                ("ㄑ", "q", "q", 1),
                ("ㄒ", "x", "x", 1),
                ("ㄓ", "zh", "z", 1),
                ("ㄔ", "ch", "c", 1),
                ("ㄕ", "sh", "s", 1),
                ("ㄖ", "r", "r", 1),
                ("ㄗ", "z", "z", 1),
                ("ㄘ", "c", "c", 1),
                ("ㄙ", "s", "s", 1),
                ("ㄧ", "i", "i", 1),
                ("ㄨ", "u", "u", 1),
                ("ㄩ", "ü", "v", 1),
                ("ㄚ", "a", "a", 1),
                ("ㄛ", "o", "o", 1),
                ("ㄜ", "e", "e", 1),
                ("ㄝ", "ê", "e", 1),
                ("ㄞ", "ai", "a", 1),
                ("ㄟ", "ei", "e", 1),
                ("ㄠ", "ao", "a", 1),
                ("ㄡ", "ou", "o", 1),
                ("ㄢ", "an", "a", 1),
                ("ㄣ", "en", "e", 1),
                ("ㄤ", "ang", "a", 1),
                ("ㄥ", "eng", "e", 1),
                ("ㄦ", "er", "e", 1),
            ];

            for (character, bopomofo, keyboard_key, layout_id) in bopomofo_chars {
                sqlx::query(
                    "INSERT INTO bopomofo_characters (`character`, bopomofo, keyboard_key, layout_id) VALUES (?, ?, ?, ?)"
                )
                .bind(character)
                .bind(bopomofo)
                .bind(keyboard_key)
                .bind(layout_id)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化繁体中文文章数据
        let traditional_chinese_text_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM traditional_chinese_texts")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if traditional_chinese_text_count == 0 {
            let traditional_chinese_texts = [
                ("基础练习", "一二人三四五六七八九十", "easy"),
                (
                    "常用句子",
                    "你好，歡迎光臨！請問有什麼可以幫您的嗎？",
                    "easy",
                ),
                (
                    "进阶练习",
                    "中華文化源遠流長，博大精深，值得我們好好學習。",
                    "medium",
                ),
                (
                    "高级练习",
                    "臺灣是一個美麗的島嶼，擁有豐富的自然資源和人文景觀。",
                    "hard",
                ),
            ];

            for (title, content, difficulty) in traditional_chinese_texts {
                sqlx::query(
                    "INSERT INTO traditional_chinese_texts (title, content, difficulty) VALUES (?, ?, ?)"
                )
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 创建日语五十音表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS japanese_gojuon (
                id INT AUTO_INCREMENT PRIMARY KEY,
                `character` VARCHAR(10) NOT NULL UNIQUE,
                romanji VARCHAR(50) NOT NULL,
                `row` VARCHAR(20) NOT NULL,
                `column` VARCHAR(20) NOT NULL,
                pronunciation VARCHAR(100) NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 创建日语汉字表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS japanese_kanji (
                id INT AUTO_INCREMENT PRIMARY KEY,
                kanji VARCHAR(10) NOT NULL UNIQUE,
                reading VARCHAR(100) NOT NULL,
                meaning VARCHAR(255) NOT NULL,
                level VARCHAR(20) NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 导入五笔字典数据（含拼音）
        let char_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wubi_characters")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if char_count == 0 {
            let dict_path = "data/wubi86_converted.json";
            if Path::new(dict_path).exists() {
                let content = fs::read_to_string(dict_path)
                    .map_err(|e| format!("Failed to read wubi dictionary: {}", e))?;
                let entries: Vec<WubiDictEntry> = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse wubi dictionary: {}", e))?;

                let entries_len = entries.len();
                println!(
                    "Importing {} wubi dictionary entries to MySQL...",
                    entries_len
                );

                for entry in entries {
                    sqlx::query(
                        "INSERT INTO wubi_characters (character_val, simple_code, full_code, pinyin, remark) VALUES (?, ?, ?, ?, ?) ON DUPLICATE KEY UPDATE simple_code=VALUES(simple_code), full_code=VALUES(full_code), pinyin=VALUES(pinyin)"
                    )
                    .bind(&entry.character)
                    .bind(&entry.simple_code)
                    .bind(&entry.full_code)
                    .bind(&entry.pinyin)
                    .bind(&entry.remark)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?;
                }

                println!("Wubi dictionary import complete: {} entries", entries_len);
            }
        }

        // 初始化日语五十音数据
        let japanese_gojuon_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM japanese_gojuon")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if japanese_gojuon_count == 0 {
            let gojuon_data = [
                ("あ", "a", "あ行", "あ段", "ア"),
                ("い", "i", "あ行", "い段", "イ"),
                ("う", "u", "あ行", "う段", "ウ"),
                ("え", "e", "あ行", "え段", "エ"),
                ("お", "o", "あ行", "お段", "オ"),
                ("か", "ka", "か行", "あ段", "カ"),
                ("き", "ki", "か行", "い段", "キ"),
                ("く", "ku", "か行", "う段", "ク"),
                ("け", "ke", "か行", "え段", "ケ"),
                ("こ", "ko", "か行", "お段", "コ"),
                ("さ", "sa", "さ行", "あ段", "サ"),
                ("し", "shi", "さ行", "い段", "シ"),
                ("す", "su", "さ行", "う段", "ス"),
                ("せ", "se", "さ行", "え段", "セ"),
                ("そ", "so", "さ行", "お段", "ソ"),
                ("た", "ta", "た行", "あ段", "タ"),
                ("ち", "chi", "た行", "い段", "チ"),
                ("つ", "tsu", "た行", "う段", "ツ"),
                ("て", "te", "た行", "え段", "テ"),
                ("と", "to", "た行", "お段", "ト"),
                ("な", "na", "な行", "あ段", "ナ"),
                ("に", "ni", "な行", "い段", "ニ"),
                ("ぬ", "nu", "な行", "う段", "ヌ"),
                ("ね", "ne", "な行", "え段", "ネ"),
                ("の", "no", "な行", "お段", "ノ"),
                ("は", "ha", "は行", "あ段", "ハ"),
                ("ひ", "hi", "は行", "い段", "ヒ"),
                ("ふ", "fu", "は行", "う段", "フ"),
                ("へ", "he", "は行", "え段", "ヘ"),
                ("ほ", "ho", "は行", "お段", "ホ"),
                ("ま", "ma", "ま行", "あ段", "マ"),
                ("み", "mi", "ま行", "い段", "ミ"),
                ("む", "mu", "ま行", "う段", "ム"),
                ("め", "me", "ま行", "え段", "メ"),
                ("も", "mo", "ま行", "お段", "モ"),
                ("や", "ya", "や行", "あ段", "ヤ"),
                ("ゆ", "yu", "や行", "う段", "ユ"),
                ("よ", "yo", "や行", "お段", "ヨ"),
                ("ら", "ra", "ら行", "あ段", "ラ"),
                ("り", "ri", "ら行", "い段", "リ"),
                ("る", "ru", "ら行", "う段", "ル"),
                ("れ", "re", "ら行", "え段", "レ"),
                ("ろ", "ro", "ら行", "お段", "ロ"),
                ("わ", "wa", "わ行", "あ段", "ワ"),
                ("を", "wo", "わ行", "お段", "ヲ"),
                ("ん", "n", "ん行", "ん段", "ン"),
            ];

            for (character, romanji, row, column, pronunciation) in gojuon_data {
                sqlx::query(
                    "INSERT INTO japanese_gojuon (`character`, romanji, `row`, `column`, pronunciation) VALUES (?, ?, ?, ?, ?)"
                )
                .bind(character)
                .bind(romanji)
                .bind(row)
                .bind(column)
                .bind(pronunciation)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化日语汉字数据
        let japanese_kanji_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM japanese_kanji")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if japanese_kanji_count == 0 {
            let kanji_data = [
                ("一", "いち", "一", "N5"),
                ("二", "に", "二", "N5"),
                ("三", "さん", "三", "N5"),
                ("四", "よん", "四", "N5"),
                ("五", "ご", "五", "N5"),
                ("六", "ろく", "六", "N5"),
                ("七", "なな", "七", "N5"),
                ("八", "はち", "八", "N5"),
                ("九", "きゅう", "九", "N5"),
                ("十", "じゅう", "十", "N5"),
                ("日", "にち", "日", "N5"),
                ("月", "つき", "月", "N5"),
                ("火", "ひ", "火", "N5"),
                ("水", "みず", "水", "N5"),
                ("木", "き", "木", "N5"),
                ("金", "かね", "金", "N5"),
                ("土", "つち", "土", "N5"),
                ("人", "ひと", "人", "N5"),
                ("大", "おお", "大", "N5"),
                ("小", "ちい", "小", "N5"),
            ];

            for (kanji, reading, meaning, level) in kanji_data {
                sqlx::query(
                    "INSERT INTO japanese_kanji (kanji, reading, meaning, level) VALUES (?, ?, ?, ?)"
                )
                .bind(kanji)
                .bind(reading)
                .bind(meaning)
                .bind(level)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    async fn get_wubi_code(&self, character: &str) -> Result<WubiCharacter, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, simple_code, full_code, pinyin FROM wubi_characters WHERE character_val = ?"
        )
        .bind(character)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Character not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, character, simple_code, full_code, pinyin)| WubiCharacter {
            id, character, simple_code, full_code, pinyin, remark: String::new()
        })
    }

    async fn get_all_wubi_characters(&self) -> Result<Vec<WubiCharacter>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String, String)>(
            "SELECT id, character_val, simple_code, full_code, pinyin, remark FROM wubi_characters ORDER BY character_val"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, simple_code, full_code, pinyin, remark)| WubiCharacter {
            id, character, simple_code, full_code, pinyin, remark
        }).collect())
    }

    async fn get_lessons(&self) -> Result<Vec<Lesson>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, character_val, code, description FROM lessons ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, character, code, description)| Lesson {
                    id,
                    character,
                    code,
                    description,
                })
                .collect()
        })
    }

    async fn get_lesson_by_id(&self, id: i32) -> Result<Lesson, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, character_val, code, description FROM lessons WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Lesson not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, character, code, description)| Lesson {
            id,
            character,
            code,
            description,
        })
    }

    async fn create_lesson(
        &self,
        character: &str,
        code: &str,
        description: &str,
    ) -> Result<Lesson, String> {
        let result =
            sqlx::query("INSERT INTO lessons (character_val, code, description) VALUES (?, ?, ?)")
                .bind(character)
                .bind(code)
                .bind(description)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        let id = result.last_insert_id() as i32;
        self.get_lesson_by_id(id).await
    }

    async fn get_articles(&self) -> Result<Vec<Article>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM articles ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, title, content, difficulty)| Article {
                    id,
                    title,
                    content,
                    difficulty,
                })
                .collect()
        })
    }

    async fn get_article_by_id(&self, id: i32) -> Result<Article, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM articles WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Article not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, title, content, difficulty)| Article {
            id,
            title,
            content,
            difficulty,
        })
    }

    async fn create_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let result =
            sqlx::query("INSERT INTO articles (title, content, difficulty) VALUES (?, ?, ?)")
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        let id = result.last_insert_id() as i32;
        self.get_article_by_id(id).await
    }

    async fn update_article(
        &self,
        id: i32,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let result =
            sqlx::query("UPDATE articles SET title = ?, content = ?, difficulty = ? WHERE id = ?")
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if result.rows_affected() == 0 {
            return Err("Article not found".to_string());
        }
        self.get_article_by_id(id).await
    }

    async fn delete_article(&self, id: i32) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM articles WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if result.rows_affected() == 0 {
            Err("Article not found".to_string())
        } else {
            Ok(())
        }
    }

    async fn get_custom_articles(&self) -> Result<Vec<Article>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM custom_articles ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, title, content, difficulty)| Article {
                    id,
                    title,
                    content,
                    difficulty,
                })
                .collect()
        })
    }

    async fn create_custom_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let result =
            sqlx::query("INSERT INTO custom_articles (title, content, difficulty) VALUES (?, ?, ?)")
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        let id = result.last_insert_id() as i32;
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM custom_articles WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|(id, title, content, difficulty)| Article {
            id,
            title,
            content,
            difficulty,
        })
    }

    async fn delete_custom_article(&self, id: i32) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM custom_articles WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if result.rows_affected() == 0 {
            Err("Custom article not found".to_string())
        } else {
            Ok(())
        }
    }

    async fn update_wubi_code(
        &self,
        character: &str,
        new_code: &str,
    ) -> Result<WubiCharacter, String> {
        let result =
            sqlx::query("UPDATE wubi_characters SET full_code = ? WHERE character_val = ?")
                .bind(new_code)
                .bind(character)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if result.rows_affected() == 0 {
            let _ = sqlx::query(
                "INSERT IGNORE INTO wubi_characters (character_val, full_code) VALUES (?, ?)",
            )
            .bind(character)
            .bind(new_code)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        Ok(WubiCharacter {
            id: 0,
            character: character.to_string(),
            simple_code: String::new(),
            full_code: new_code.to_string(),
            pinyin: String::new(),
            remark: String::new(),
        })
    }

    async fn get_wubi_roots(&self) -> Result<Vec<WubiRoot>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots ORDER BY position"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, code, position, description)| WubiRoot {
            id, character, code, position, description
        }).collect())
    }

    async fn get_wubi_root_by_id(&self, id: i32) -> Result<WubiRoot, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots WHERE id = ?",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Wubi root not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, character, code, position, description)| WubiRoot {
            id,
            character,
            code,
            position,
            description,
        })
    }

    async fn create_wubi_root(
        &self,
        character: &str,
        code: &str,
        position: &str,
        description: &str,
    ) -> Result<WubiRoot, String> {
        let result = sqlx::query(
            "INSERT INTO wubi_roots (character_val, code, position, description) VALUES (?, ?, ?, ?)"
        )
        .bind(character)
        .bind(code)
        .bind(position)
        .bind(description)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let id = result.last_insert_id() as i32;
        self.get_wubi_root_by_id(id).await
    }

    async fn search_wubi_root(&self, character: &str) -> Result<Option<WubiRoot>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots WHERE character_val = ?"
        )
        .bind(character)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(
            result.map(|(id, character, code, position, description)| WubiRoot {
                id,
                character,
                code,
                position,
                description,
            }),
        )
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String> {
        let result = sqlx::query_as::<
            _,
            (i32, String, String, String, chrono::DateTime<chrono::Utc>),
        >(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE username = ?",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(
            result.map(|(id, username, email, password_hash, created_at)| User {
                id,
                username,
                email,
                password_hash,
                created_at: created_at.to_rfc3339(),
            }),
        )
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
        let result =
            sqlx::query_as::<_, (i32, String, String, String, chrono::DateTime<chrono::Utc>)>(
                "SELECT id, username, email, password_hash, created_at FROM users WHERE id = ?",
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(
            result.map(|(id, username, email, password_hash, created_at)| User {
                id,
                username,
                email,
                password_hash,
                created_at: created_at.to_rfc3339(),
            }),
        )
    }

    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, String> {
        let result =
            sqlx::query("INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)")
                .bind(username)
                .bind(email)
                .bind(password_hash)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        let id = result.last_insert_id() as i32;
        self.get_user_by_id(id)
            .await
            .map(|opt| opt.expect("User should exist after creation"))
    }

    async fn save_progress(
        &self,
        user_name: &str,
        lesson_id: i32,
        accuracy: f32,
        score: i32,
    ) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO user_progress (user_name, lesson_id, accuracy, score) VALUES (?, ?, ?, ?)",
        )
        .bind(user_name)
        .bind(lesson_id)
        .bind(accuracy)
        .bind(score)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn get_key_radicals(&self) -> Result<Vec<KeyRadical>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, key_char, radicals, formula, description FROM key_radicals ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(
                    |(id, key_char, radicals, formula, description)| KeyRadical {
                        id,
                        key_char,
                        radicals,
                        formula,
                        description,
                    },
                )
                .collect()
        })
    }

    async fn get_key_radical_by_key(&self, key_char: &str) -> Result<Option<KeyRadical>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, key_char, radicals, formula, description FROM key_radicals WHERE key_char = ?"
        )
        .bind(key_char)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.map(
            |(id, key_char, radicals, formula, description)| KeyRadical {
                id,
                key_char,
                radicals,
                formula,
                description,
            },
        ))
    }

    async fn get_english_texts(&self) -> Result<Vec<EnglishText>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM english_texts ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, title, content, difficulty)| EnglishText {
                    id,
                    title,
                    content,
                    difficulty,
                })
                .collect()
        })
    }

    async fn get_japanese_texts(&self) -> Result<Vec<JapaneseText>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, title, content, difficulty, type FROM japanese_texts ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, title, content, difficulty, r#type)| JapaneseText {
                    id,
                    title,
                    content,
                    difficulty,
                    r#type,
                })
                .collect()
        })
    }

    async fn get_japanese_keyboards(&self) -> Result<Vec<JapaneseKeyboard>, String> {
        sqlx::query_as::<_, (i32, String, String)>(
            "SELECT id, layout_name, description FROM japanese_keyboards ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, layout_name, description)| JapaneseKeyboard {
                    id,
                    layout_name,
                    description,
                })
                .collect()
        })
    }

    async fn get_japanese_characters(&self) -> Result<Vec<JapaneseCharacter>, String> {
        sqlx::query_as::<_, (i32, String, String, String, i32)>(
            "SELECT id, `character`, romanji, keyboard_key, layout_id FROM japanese_characters ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, romanji, keyboard_key, layout_id)| JapaneseCharacter {
            id, character, romanji, keyboard_key, layout_id
        }).collect())
    }

    async fn get_traditional_chinese_texts(&self) -> Result<Vec<TraditionalChineseText>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM traditional_chinese_texts ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, title, content, difficulty)| TraditionalChineseText {
                    id,
                    title,
                    content,
                    difficulty,
                })
                .collect()
        })
    }

    async fn get_bopomofo_keyboards(&self) -> Result<Vec<BopomofoKeyboard>, String> {
        sqlx::query_as::<_, (i32, String, String)>(
            "SELECT id, layout_name, description FROM bopomofo_keyboards ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, layout_name, description)| BopomofoKeyboard {
                    id,
                    layout_name,
                    description,
                })
                .collect()
        })
    }

    async fn get_bopomofo_characters(&self) -> Result<Vec<BopomofoCharacter>, String> {
        sqlx::query_as::<_, (i32, String, String, String, i32)>(
            "SELECT id, `character`, bopomofo, keyboard_key, layout_id FROM bopomofo_characters ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, bopomofo, keyboard_key, layout_id)| BopomofoCharacter {
            id, character, bopomofo, keyboard_key, layout_id
        }).collect())
    }

    async fn get_japanese_gojuon(&self) -> Result<Vec<JapaneseGojuon>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String, String)>(
            "SELECT id, \"character\", romanji, \"row\", \"column\", pronunciation FROM japanese_gojuon ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, romanji, row, column, pronunciation)| JapaneseGojuon {
            id, character, romanji, row, column, pronunciation
        }).collect())
    }

    async fn get_japanese_kanji(&self) -> Result<Vec<JapaneseKanji>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, kanji, reading, meaning, level FROM japanese_kanji ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, kanji, reading, meaning, level)| JapaneseKanji {
                    id,
                    kanji,
                    reading,
                    meaning,
                    level,
                })
                .collect()
        })
    }

    async fn get_japanese_kanji_by_char(
        &self,
        kanji: &str,
    ) -> Result<Option<JapaneseKanji>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, kanji, reading, meaning, level FROM japanese_kanji WHERE kanji = $1",
        )
        .bind(kanji)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(
            result.map(|(id, kanji, reading, meaning, level)| JapaneseKanji {
                id,
                kanji,
                reading,
                meaning,
                level,
            }),
        )
    }
}

/// Redis数据库实现
pub struct RedisDatabase {
    client: redis::Client,
}

impl RedisDatabase {
    pub fn new(url: &str) -> Result<Self, String> {
        let client = redis::Client::open(url)
            .map_err(|e| format!("Failed to create Redis client: {}", e))?;
        Ok(Self { client })
    }

    async fn get_connection(&self) -> Result<redis::aio::MultiplexedConnection, String> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| format!("Failed to get Redis connection: {}", e))
    }

    async fn import_wubi_dict(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
    ) -> Result<(), String> {
        let dict_path = "data/wubi_dict.json";

        if !Path::new(dict_path).exists() {
            return Err(format!("Wubi dictionary file not found: {}", dict_path));
        }

        let content = fs::read_to_string(dict_path)
            .map_err(|e| format!("Failed to read wubi dictionary file: {}", e))?;

        let entries: Vec<WubiDictEntry> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse wubi dictionary JSON: {}", e))?;

        println!(
            "Importing {} wubi dictionary entries to Redis...",
            entries.len()
        );

        let batch_size = 100;
        for chunk in entries.chunks(batch_size) {
            let mut pipe = redis::pipe();
            for entry in chunk {
                pipe.set(&format!("wubi:char:{}", entry.character), &entry.full_code);
            }
            let _: () = pipe
                .query_async(conn)
                .await
                .map_err(|e| format!("Failed to batch insert wubi characters: {}", e))?;
        }

        println!("Wubi dictionary import complete: {} entries", entries.len());
        Ok(())
    }

    async fn import_sample_data(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
    ) -> Result<(), String> {
        let _: () = conn
            .incr("wubi:id:lessons", 3)
            .await
            .map_err(|e| format!("Failed to set lesson id counter: {}", e))?;

        let lessons = [
            ("1", "人", "WG", "练习人字的五笔编码。"),
            ("2", "日", "KH", "练习日字的五笔编码。"),
            ("3", "山", "FQ", "练习山字的五笔编码。"),
        ];

        for (id, char, code, desc) in lessons {
            let _: () = conn
                .hset_multiple(
                    &format!("wubi:lesson:{}", id),
                    &[
                        ("id", id),
                        ("character", char),
                        ("code", code),
                        ("description", desc),
                    ],
                )
                .await
                .map_err(|e| format!("Failed to insert lesson: {}", e))?;

            let _: () = conn
                .sadd("wubi:lessons", id)
                .await
                .map_err(|e| format!("Failed to add lesson to set: {}", e))?;
        }

        let _: () = conn
            .incr("wubi:id:articles", 3)
            .await
            .map_err(|e| format!("Failed to set article id counter: {}", e))?;

        let articles = [
            (
                "1",
                "练习文章一",
                "五笔字型是一种高效的中文输入法，通过拆分汉字为基本字根进行输入。",
                "easy",
            ),
            (
                "2",
                "练习文章二",
                "学习五笔需要掌握字根分布和拆字规则，多加练习才能熟练运用。",
                "medium",
            ),
            (
                "3",
                "练习文章三",
                "汉字的结构复杂多样，五笔输入法按照汉字的笔画和结构规律进行编码。",
                "hard",
            ),
        ];

        for (id, title, content, difficulty) in articles {
            let _: () = conn
                .hset_multiple(
                    &format!("wubi:article:{}", id),
                    &[
                        ("id", id),
                        ("title", title),
                        ("content", content),
                        ("difficulty", difficulty),
                    ],
                )
                .await
                .map_err(|e| format!("Failed to insert article: {}", e))?;

            let _: () = conn
                .sadd("wubi:articles", id)
                .await
                .map_err(|e| format!("Failed to add article to set: {}", e))?;
        }

        let _: () = conn
            .incr("wubi:id:roots", 10)
            .await
            .map_err(|e| format!("Failed to set root id counter: {}", e))?;

        let roots = [
            ("1", "一", "GGLL", "G区第一键", "横区起首字根"),
            ("2", "丿", "TTLL", "T区第一键", "撇区起首字根"),
            ("3", "丨", "HHLL", "H区第一键", "竖区起首字根"),
            ("4", "丶", "YYLL", "Y区第一键", "捺区起首字根"),
            ("5", "乙", "NNLL", "N区第一键", "折区起首字根"),
            ("6", "九", "VTNG", "V区第二键", "字根：乙"),
            ("7", "力", "LTNN", "L键", "字根：力"),
            ("8", "乃", "DETN", "N键", "字根：乃"),
            ("9", "刀", "VNTE", "V键", "字根：刀"),
            ("10", "卜", "HHYD", "H键", "字根：卜"),
        ];

        for (id, char, code, position, desc) in roots {
            let _: () = conn
                .hset_multiple(
                    &format!("wubi:root:{}", id),
                    &[
                        ("id", id),
                        ("character", char),
                        ("code", code),
                        ("position", position),
                        ("description", desc),
                    ],
                )
                .await
                .map_err(|e| format!("Failed to insert root: {}", e))?;

            let _: () = conn
                .sadd("wubi:roots", id)
                .await
                .map_err(|e| format!("Failed to add root to set: {}", e))?;
        }

        println!("Sample data imported to Redis");
        Ok(())
    }
}

#[async_trait]
impl Database for RedisDatabase {
    async fn init_db(&self) -> Result<(), String> {
        let mut conn = self.get_connection().await?;

        let _: () = conn
            .set("wubi:initialized", "true")
            .await
            .map_err(|e| format!("Failed to initialize Redis: {}", e))?;

        let initialized: Option<String> = conn
            .get("wubi:data_imported")
            .await
            .map_err(|e| format!("Failed to check import status: {}", e))?;

        if initialized.is_none() {
            self.import_wubi_dict(&mut conn).await?;
            self.import_sample_data(&mut conn).await?;

            let _: () = conn
                .set("wubi:data_imported", "true")
                .await
                .map_err(|e| format!("Failed to set import flag: {}", e))?;
        }

        Ok(())
    }

    async fn get_wubi_code(&self, character: &str) -> Result<WubiCharacter, String> {
        let mut conn = self.get_connection().await?;

        let code: Option<String> = conn
            .get(&format!("wubi:char:{}", character))
            .await
            .map_err(|e| format!("Failed to get wubi code: {}", e))?;

        match code {
            Some(wubi_code) => Ok(WubiCharacter {
                id: 0,
                character: character.to_string(),
                simple_code: wubi_code.clone(),
                full_code: wubi_code,
                pinyin: String::new(),
                remark: String::new(),
            }),
            None => Err("Character not found".to_string()),
        }
    }

    async fn get_all_wubi_characters(&self) -> Result<Vec<WubiCharacter>, String> {
        let mut conn = self.get_connection().await?;

        let keys: Vec<String> = conn
            .keys("wubi:char:*")
            .await
            .map_err(|e| format!("Failed to get keys: {}", e))?;

        let mut characters = Vec::new();
        for key in keys {
            let wubi_code: Option<String> = conn
                .get(&key)
                .await
                .map_err(|e| format!("Failed to get value: {}", e))?;

            if let Some(code) = wubi_code {
                let character = key.strip_prefix("wubi:char:").unwrap_or("").to_string();
                characters.push(WubiCharacter {
                    id: 0,
                    character,
                    simple_code: code.clone(),
                    full_code: code,
                    pinyin: String::new(),
                    remark: String::new(),
                });
            }
        }

        characters.sort_by(|a, b| a.character.cmp(&b.character));
        Ok(characters)
    }

    async fn get_lessons(&self) -> Result<Vec<Lesson>, String> {
        let mut conn = self.get_connection().await?;

        let lesson_ids: Vec<String> = conn
            .smembers("wubi:lessons")
            .await
            .map_err(|e| format!("Failed to get lesson ids: {}", e))?;

        let mut lessons = Vec::new();
        for id in lesson_ids {
            let character: String = conn
                .hget(&format!("wubi:lesson:{}", id), "character")
                .await
                .map_err(|e| format!("Failed to get character: {}", e))?;
            let code: String = conn
                .hget(&format!("wubi:lesson:{}", id), "code")
                .await
                .map_err(|e| format!("Failed to get code: {}", e))?;
            let description: String = conn
                .hget(&format!("wubi:lesson:{}", id), "description")
                .await
                .map_err(|e| format!("Failed to get description: {}", e))?;

            lessons.push(Lesson {
                id: id.parse().unwrap_or(0),
                character,
                code,
                description,
            });
        }

        lessons.sort_by_key(|l| l.id);
        Ok(lessons)
    }

    async fn get_lesson_by_id(&self, id: i32) -> Result<Lesson, String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:lesson:{}", id);

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| format!("Failed to check lesson: {}", e))?;

        if !exists {
            return Err("Lesson not found".to_string());
        }

        let character: String = conn
            .hget(&key, "character")
            .await
            .map_err(|e| format!("Failed to get character: {}", e))?;
        let code: String = conn
            .hget(&key, "code")
            .await
            .map_err(|e| format!("Failed to get code: {}", e))?;
        let description: String = conn
            .hget(&key, "description")
            .await
            .map_err(|e| format!("Failed to get description: {}", e))?;

        Ok(Lesson {
            id,
            character,
            code,
            description,
        })
    }

    async fn create_lesson(
        &self,
        character: &str,
        code: &str,
        description: &str,
    ) -> Result<Lesson, String> {
        let mut conn = self.get_connection().await?;

        let id: i32 = conn
            .incr("wubi:id:lessons", 1)
            .await
            .map_err(|e| format!("Failed to generate id: {}", e))?;

        let key = format!("wubi:lesson:{}", id);
        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("id", id.to_string()),
                    ("character", character.to_string()),
                    ("code", code.to_string()),
                    ("description", description.to_string()),
                ],
            )
            .await
            .map_err(|e| format!("Failed to create lesson: {}", e))?;

        let _: () = conn
            .sadd("wubi:lessons", id.to_string())
            .await
            .map_err(|e| format!("Failed to add lesson to set: {}", e))?;

        Ok(Lesson {
            id,
            character: character.to_string(),
            code: code.to_string(),
            description: description.to_string(),
        })
    }

    async fn get_articles(&self) -> Result<Vec<Article>, String> {
        let mut conn = self.get_connection().await?;

        let article_ids: Vec<String> = conn
            .smembers("wubi:articles")
            .await
            .map_err(|e| format!("Failed to get article ids: {}", e))?;

        let mut articles = Vec::new();
        for id in article_ids {
            let title: String = conn
                .hget(&format!("wubi:article:{}", id), "title")
                .await
                .map_err(|e| format!("Failed to get title: {}", e))?;
            let content: String = conn
                .hget(&format!("wubi:article:{}", id), "content")
                .await
                .map_err(|e| format!("Failed to get content: {}", e))?;
            let difficulty: String = conn
                .hget(&format!("wubi:article:{}", id), "difficulty")
                .await
                .map_err(|e| format!("Failed to get difficulty: {}", e))?;

            articles.push(Article {
                id: id.parse().unwrap_or(0),
                title,
                content,
                difficulty,
            });
        }

        articles.sort_by_key(|a| a.id);
        Ok(articles)
    }

    async fn get_article_by_id(&self, id: i32) -> Result<Article, String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:article:{}", id);

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| format!("Failed to check article: {}", e))?;

        if !exists {
            return Err("Article not found".to_string());
        }

        let title: String = conn
            .hget(&key, "title")
            .await
            .map_err(|e| format!("Failed to get title: {}", e))?;
        let content: String = conn
            .hget(&key, "content")
            .await
            .map_err(|e| format!("Failed to get content: {}", e))?;
        let difficulty: String = conn
            .hget(&key, "difficulty")
            .await
            .map_err(|e| format!("Failed to get difficulty: {}", e))?;

        Ok(Article {
            id,
            title,
            content,
            difficulty,
        })
    }

    async fn create_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let mut conn = self.get_connection().await?;

        let id: i32 = conn
            .incr("wubi:id:articles", 1)
            .await
            .map_err(|e| format!("Failed to generate id: {}", e))?;

        let key = format!("wubi:article:{}", id);
        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("id", id.to_string()),
                    ("title", title.to_string()),
                    ("content", content.to_string()),
                    ("difficulty", difficulty.to_string()),
                ],
            )
            .await
            .map_err(|e| format!("Failed to create article: {}", e))?;

        let _: () = conn
            .sadd("wubi:articles", id.to_string())
            .await
            .map_err(|e| format!("Failed to add article to set: {}", e))?;

        Ok(Article {
            id,
            title: title.to_string(),
            content: content.to_string(),
            difficulty: difficulty.to_string(),
        })
    }

    async fn update_article(
        &self,
        id: i32,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:article:{}", id);

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| format!("Failed to check article: {}", e))?;

        if !exists {
            return Err("Article not found".to_string());
        }

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("title", title.to_string()),
                    ("content", content.to_string()),
                    ("difficulty", difficulty.to_string()),
                ],
            )
            .await
            .map_err(|e| format!("Failed to update article: {}", e))?;

        Ok(Article {
            id,
            title: title.to_string(),
            content: content.to_string(),
            difficulty: difficulty.to_string(),
        })
    }

    async fn delete_article(&self, id: i32) -> Result<(), String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:article:{}", id);

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| format!("Failed to check article: {}", e))?;

        if !exists {
            return Err("Article not found".to_string());
        }

        let _: () = conn
            .del(&key)
            .await
            .map_err(|e| format!("Failed to delete article: {}", e))?;
        let _: () = conn
            .srem("wubi:articles", id.to_string())
            .await
            .map_err(|e| format!("Failed to remove from set: {}", e))?;

        Ok(())
    }

    async fn get_custom_articles(&self) -> Result<Vec<Article>, String> {
        let mut conn = self.get_connection().await?;

        let article_ids: Vec<String> = conn
            .smembers("wubi:custom:articles")
            .await
            .map_err(|e| format!("Failed to get custom article ids: {}", e))?;

        let mut articles = Vec::new();
        for id in article_ids {
            let title: String = conn
                .hget(&format!("wubi:custom:article:{}", id), "title")
                .await
                .map_err(|e| format!("Failed to get title: {}", e))?;
            let content: String = conn
                .hget(&format!("wubi:custom:article:{}", id), "content")
                .await
                .map_err(|e| format!("Failed to get content: {}", e))?;
            let difficulty: String = conn
                .hget(&format!("wubi:custom:article:{}", id), "difficulty")
                .await
                .map_err(|e| format!("Failed to get difficulty: {}", e))?;

            articles.push(Article {
                id: id.parse().unwrap_or(0),
                title,
                content,
                difficulty,
            });
        }

        articles.sort_by_key(|a| a.id);
        Ok(articles)
    }

    async fn create_custom_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let mut conn = self.get_connection().await?;

        let id: i32 = conn
            .incr("wubi:id:custom:articles", 1)
            .await
            .map_err(|e| format!("Failed to generate custom article id: {}", e))?;

        let key = format!("wubi:custom:article:{}", id);
        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("id", id.to_string()),
                    ("title", title.to_string()),
                    ("content", content.to_string()),
                    ("difficulty", difficulty.to_string()),
                ],
            )
            .await
            .map_err(|e| format!("Failed to create custom article: {}", e))?;

        let _: () = conn
            .sadd("wubi:custom:articles", id.to_string())
            .await
            .map_err(|e| format!("Failed to add custom article to set: {}", e))?;

        Ok(Article {
            id,
            title: title.to_string(),
            content: content.to_string(),
            difficulty: difficulty.to_string(),
        })
    }

    async fn delete_custom_article(&self, id: i32) -> Result<(), String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:custom:article:{}", id);

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| format!("Failed to check custom article: {}", e))?;

        if !exists {
            return Err("Custom article not found".to_string());
        }

        let _: () = conn
            .del(&key)
            .await
            .map_err(|e| format!("Failed to delete custom article: {}", e))?;
        let _: () = conn
            .srem("wubi:custom:articles", id.to_string())
            .await
            .map_err(|e| format!("Failed to remove custom article from set: {}", e))?;

        Ok(())
    }

    async fn update_wubi_code(
        &self,
        character: &str,
        new_code: &str,
    ) -> Result<WubiCharacter, String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:char:{}", character);

        let _: () = conn
            .set(&key, new_code)
            .await
            .map_err(|e| format!("Failed to update wubi code: {}", e))?;

        Ok(WubiCharacter {
            id: 0,
            character: character.to_string(),
            simple_code: String::new(),
            full_code: new_code.to_string(),
            pinyin: String::new(),
            remark: String::new(),
        })
    }

    async fn get_wubi_roots(&self) -> Result<Vec<WubiRoot>, String> {
        let mut conn = self.get_connection().await?;

        let root_ids: Vec<String> = conn
            .smembers("wubi:roots")
            .await
            .map_err(|e| format!("Failed to get root ids: {}", e))?;

        let mut roots = Vec::new();
        for id in root_ids {
            let character: String = conn
                .hget(&format!("wubi:root:{}", id), "character")
                .await
                .map_err(|e| format!("Failed to get character: {}", e))?;
            let code: String = conn
                .hget(&format!("wubi:root:{}", id), "code")
                .await
                .map_err(|e| format!("Failed to get code: {}", e))?;
            let position: String = conn
                .hget(&format!("wubi:root:{}", id), "position")
                .await
                .map_err(|e| format!("Failed to get position: {}", e))?;
            let description: String = conn
                .hget(&format!("wubi:root:{}", id), "description")
                .await
                .map_err(|e| format!("Failed to get description: {}", e))?;

            roots.push(WubiRoot {
                id: id.parse().unwrap_or(0),
                character,
                code,
                position,
                description,
            });
        }

        roots.sort_by_key(|r| r.position.clone());
        Ok(roots)
    }

    async fn get_wubi_root_by_id(&self, id: i32) -> Result<WubiRoot, String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:root:{}", id);

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| format!("Failed to check root: {}", e))?;

        if !exists {
            return Err("Wubi root not found".to_string());
        }

        let character: String = conn
            .hget(&key, "character")
            .await
            .map_err(|e| format!("Failed to get character: {}", e))?;
        let code: String = conn
            .hget(&key, "code")
            .await
            .map_err(|e| format!("Failed to get code: {}", e))?;
        let position: String = conn
            .hget(&key, "position")
            .await
            .map_err(|e| format!("Failed to get position: {}", e))?;
        let description: String = conn
            .hget(&key, "description")
            .await
            .map_err(|e| format!("Failed to get description: {}", e))?;

        Ok(WubiRoot {
            id,
            character,
            code,
            position,
            description,
        })
    }

    async fn create_wubi_root(
        &self,
        character: &str,
        code: &str,
        position: &str,
        description: &str,
    ) -> Result<WubiRoot, String> {
        let mut conn = self.get_connection().await?;

        let id: i32 = conn
            .incr("wubi:id:roots", 1)
            .await
            .map_err(|e| format!("Failed to generate id: {}", e))?;

        let key = format!("wubi:root:{}", id);
        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("id", id.to_string()),
                    ("character", character.to_string()),
                    ("code", code.to_string()),
                    ("position", position.to_string()),
                    ("description", description.to_string()),
                ],
            )
            .await
            .map_err(|e| format!("Failed to create root: {}", e))?;

        let _: () = conn
            .sadd("wubi:roots", id.to_string())
            .await
            .map_err(|e| format!("Failed to add root to set: {}", e))?;

        Ok(WubiRoot {
            id,
            character: character.to_string(),
            code: code.to_string(),
            position: position.to_string(),
            description: description.to_string(),
        })
    }

    async fn search_wubi_root(&self, character: &str) -> Result<Option<WubiRoot>, String> {
        let mut conn = self.get_connection().await?;

        let root_ids: Vec<String> = conn
            .smembers("wubi:roots")
            .await
            .map_err(|e| format!("Failed to get root ids: {}", e))?;

        for id in root_ids {
            let stored_char: String = conn
                .hget(&format!("wubi:root:{}", id), "character")
                .await
                .map_err(|e| format!("Failed to get character: {}", e))?;

            if stored_char == character {
                let code: String = conn
                    .hget(&format!("wubi:root:{}", id), "code")
                    .await
                    .map_err(|e| format!("Failed to get code: {}", e))?;
                let position: String = conn
                    .hget(&format!("wubi:root:{}", id), "position")
                    .await
                    .map_err(|e| format!("Failed to get position: {}", e))?;
                let description: String = conn
                    .hget(&format!("wubi:root:{}", id), "description")
                    .await
                    .map_err(|e| format!("Failed to get description: {}", e))?;

                return Ok(Some(WubiRoot {
                    id: id.parse().unwrap_or(0),
                    character: stored_char,
                    code,
                    position,
                    description,
                }));
            }
        }

        Ok(None)
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String> {
        let mut conn = self.get_connection().await?;
        let key = format!("wubi:user:{}", username);

        let exists: bool = conn
            .exists(&key)
            .await
            .map_err(|e| format!("Failed to check user: {}", e))?;

        if !exists {
            return Ok(None);
        }

        let id: String = conn
            .hget(&key, "id")
            .await
            .map_err(|e| format!("Failed to get id: {}", e))?;
        let email: String = conn
            .hget(&key, "email")
            .await
            .map_err(|e| format!("Failed to get email: {}", e))?;
        let password_hash: String = conn
            .hget(&key, "password_hash")
            .await
            .map_err(|e| format!("Failed to get password_hash: {}", e))?;
        let created_at: String = conn
            .hget(&key, "created_at")
            .await
            .map_err(|e| format!("Failed to get created_at: {}", e))?;

        Ok(Some(User {
            id: id.parse().unwrap_or(0),
            username: username.to_string(),
            email,
            password_hash,
            created_at,
        }))
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
        let mut conn = self.get_connection().await?;

        let username: Option<String> = conn
            .get(&format!("wubi:userid:{}:username", id))
            .await
            .map_err(|e| format!("Failed to get username mapping: {}", e))?;

        match username {
            Some(username) => self.get_user_by_username(&username).await,
            None => Ok(None),
        }
    }

    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, String> {
        let mut conn = self.get_connection().await?;

        let id: i32 = conn
            .incr("wubi:id:users", 1)
            .await
            .map_err(|e| format!("Failed to generate id: {}", e))?;

        let now = chrono::Utc::now().to_rfc3339();
        let key = format!("wubi:user:{}", username);

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("id", id.to_string()),
                    ("username", username.to_string()),
                    ("email", email.to_string()),
                    ("password_hash", password_hash.to_string()),
                    ("created_at", now.clone()),
                ],
            )
            .await
            .map_err(|e| format!("Failed to create user: {}", e))?;

        let _: () = conn
            .set(&format!("wubi:userid:{}:username", id), username)
            .await
            .map_err(|e| format!("Failed to create user id mapping: {}", e))?;

        Ok(User {
            id,
            username: username.to_string(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            created_at: now,
        })
    }

    async fn save_progress(
        &self,
        user_name: &str,
        lesson_id: i32,
        accuracy: f32,
        score: i32,
    ) -> Result<(), String> {
        let mut conn = self.get_connection().await?;

        let key = format!("wubi:progress:{}:{}", user_name, lesson_id);
        let now = chrono::Utc::now().to_rfc3339();

        let _: () = conn
            .hset_multiple(
                &key,
                &[
                    ("user_name", user_name.to_string()),
                    ("lesson_id", lesson_id.to_string()),
                    ("accuracy", accuracy.to_string()),
                    ("score", score.to_string()),
                    ("updated_at", now),
                ],
            )
            .await
            .map_err(|e| format!("Failed to save progress: {}", e))?;

        Ok(())
    }

    async fn get_key_radicals(&self) -> Result<Vec<KeyRadical>, String> {
        let mut conn = self.get_connection().await?;

        let keys: Vec<String> = conn
            .keys("wubi:key_radical:*")
            .await
            .map_err(|e| format!("Failed to get keys: {}", e))?;

        let mut results = Vec::new();
        for key in keys {
            let radicals: Option<String> = conn.hget(&key, "radicals").await.unwrap_or(None);
            let formula: Option<String> = conn.hget(&key, "formula").await.unwrap_or(None);
            let description: Option<String> = conn.hget(&key, "description").await.unwrap_or(None);
            let key_char = key
                .strip_prefix("wubi:key_radical:")
                .unwrap_or("")
                .to_string();

            if let Some(radicals) = radicals {
                let id: i32 = key.split(':').last().unwrap_or("0").parse().unwrap_or(0);
                results.push(KeyRadical {
                    id,
                    key_char,
                    radicals,
                    formula: formula.unwrap_or_default(),
                    description: description.unwrap_or_default(),
                });
            }
        }

        results.sort_by_key(|r| r.key_char.clone());
        Ok(results)
    }

    async fn get_key_radical_by_key(&self, key_char: &str) -> Result<Option<KeyRadical>, String> {
        let mut conn = self.get_connection().await?;

        let key = format!("wubi:key_radical:{}", key_char);
        let radicals: Option<String> = conn
            .hget(&key, "radicals")
            .await
            .map_err(|e| format!("Failed to get radicals: {}", e))?;
        let formula: Option<String> = conn
            .hget(&key, "formula")
            .await
            .map_err(|e| format!("Failed to get formula: {}", e))?;
        let description: Option<String> = conn
            .hget(&key, "description")
            .await
            .map_err(|e| format!("Failed to get description: {}", e))?;

        match radicals {
            Some(radicals) => Ok(Some(KeyRadical {
                id: 0,
                key_char: key_char.to_string(),
                radicals,
                formula: formula.unwrap_or_default(),
                description: description.unwrap_or_default(),
            })),
            None => Ok(None),
        }
    }

    async fn get_english_texts(&self) -> Result<Vec<EnglishText>, String> {
        Ok(vec![])
    }

    async fn get_japanese_texts(&self) -> Result<Vec<JapaneseText>, String> {
        Ok(vec![])
    }

    async fn get_japanese_keyboards(&self) -> Result<Vec<JapaneseKeyboard>, String> {
        Ok(vec![])
    }

    async fn get_japanese_characters(&self) -> Result<Vec<JapaneseCharacter>, String> {
        Ok(vec![])
    }

    async fn get_traditional_chinese_texts(&self) -> Result<Vec<TraditionalChineseText>, String> {
        Ok(vec![])
    }

    async fn get_bopomofo_keyboards(&self) -> Result<Vec<BopomofoKeyboard>, String> {
        Ok(vec![])
    }

    async fn get_bopomofo_characters(&self) -> Result<Vec<BopomofoCharacter>, String> {
        Ok(vec![])
    }

    async fn get_japanese_gojuon(&self) -> Result<Vec<JapaneseGojuon>, String> {
        Ok(vec![])
    }

    async fn get_japanese_kanji(&self) -> Result<Vec<JapaneseKanji>, String> {
        Ok(vec![])
    }

    async fn get_japanese_kanji_by_char(
        &self,
        kanji: &str,
    ) -> Result<Option<JapaneseKanji>, String> {
        Ok(None)
    }
}

/// MongoDB数据库实现
pub struct MongoDatabase {
    client: Client,
    database: String,
}

impl MongoDatabase {
    pub async fn new(uri: &str) -> Result<Self, String> {
        let client = Client::with_uri_str(uri)
            .await
            .map_err(|e| format!("Failed to create MongoDB client: {}", e))?;
        Ok(Self {
            client,
            database: "wubi".to_string(),
        })
    }

    fn get_db(&self) -> MongoDb {
        self.client.database(&self.database)
    }

    async fn import_wubi_dict(&self) -> Result<(), String> {
        let dict_path = "data/wubi86_converted.json";
        if !Path::new(dict_path).exists() {
            return Err(format!("Wubi dictionary file not found: {}", dict_path));
        }
        let content = fs::read_to_string(dict_path)
            .map_err(|e| format!("Failed to read wubi dictionary file: {}", e))?;
        let entries: Vec<WubiDictEntry> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse wubi dictionary JSON: {}", e))?;
        let db = self.get_db();
        let chars_collection = db.collection::<Document>("wubi_characters");
        let entries_len = entries.len();
        println!(
            "Importing {} wubi dictionary entries to MongoDB...",
            entries_len
        );

        // Clear existing data before import
        chars_collection
            .delete_many(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to clear wubi characters: {}", e))?;

        // Batch insert to avoid BSON size limit (16MB)
        const BATCH_SIZE: usize = 5000;
        let mut docs = Vec::with_capacity(BATCH_SIZE);
        let mut inserted = 0usize;
        for entry in entries {
            docs.push(doc! {
                "character": entry.character,
                "simple_code": entry.simple_code,
                "full_code": entry.full_code,
                "pinyin": entry.pinyin,
                "remark": entry.remark,
            });
            if docs.len() >= BATCH_SIZE {
                let count = docs.len();
                chars_collection
                    .insert_many(docs, None)
                    .await
                    .map_err(|e| format!("Failed to insert wubi characters batch: {}", e))?;
                inserted += count;
                println!("  Inserted {}/{} wubi characters...", inserted, entries_len);
                docs = Vec::with_capacity(BATCH_SIZE);
            }
        }
        if !docs.is_empty() {
            let count = docs.len();
            chars_collection
                .insert_many(docs, None)
                .await
                .map_err(|e| format!("Failed to insert final wubi characters batch: {}", e))?;
            inserted += count;
        }

        // Create index on character field for fast lookup (non-unique because some characters may repeat)
        let index_model = mongodb::IndexModel::builder()
            .keys(doc! { "character": 1 })
            .build();
        chars_collection
            .create_index(index_model, None)
            .await
            .map_err(|e| format!("Failed to create index on wubi_characters: {}", e))?;

        println!("Wubi dictionary import complete: {} entries", inserted);
        Ok(())
    }

    async fn import_sample_data(&self) -> Result<(), String> {
        let db = self.get_db();

        // --- lessons ---
        let lessons_col = db.collection::<Document>("lessons");
        lessons_col.delete_many(doc! {}, None).await.ok();
        let lessons_docs: Vec<Document> = [
            ("人", "W", "单人旁，常用字根"),
            ("口", "K", "口字旁，常用字根"),
            ("日", "J", "日字旁，常用字根"),
        ]
        .iter()
        .map(|(c, code, desc)| doc! { "character": *c, "code": *code, "description": *desc })
        .collect();
        lessons_col.insert_many(lessons_docs, None).await.map_err(|e| format!("Failed to insert lessons: {}", e))?;

        // --- articles ---
        let articles_col = db.collection::<Document>("articles");
        articles_col.delete_many(doc! {}, None).await.ok();
        let articles_docs: Vec<Document> = [
            ("练习文章一", "五笔字型是一种高效的中文输入法，通过拆分汉字为基本字根进行输入。", "easy"),
            ("练习文章二", "学习五笔需要掌握字根分布和拆字规则，多加练习才能熟练运用。", "medium"),
            ("练习文章三", "汉字的结构复杂多样，五笔输入法按照汉字的笔画和结构规律进行编码。", "hard"),
        ]
        .iter()
        .map(|(t, c, d)| doc! { "title": *t, "content": *c, "difficulty": *d })
        .collect();
        articles_col.insert_many(articles_docs, None).await.map_err(|e| format!("Failed to insert articles: {}", e))?;

        // --- wubi_roots ---
        let roots_col = db.collection::<Document>("wubi_roots");
        roots_col.delete_many(doc! {}, None).await.ok();
        let roots_docs: Vec<Document> = [
            ("一", "GGLL", "G区第一键", "横区起首字根"),
            ("丿", "TTLL", "T区第一键", "撇区起首字根"),
            ("丨", "HHLL", "H区第一键", "竖区起首字根"),
            ("丶", "YYLL", "Y区第一键", "捺区起首字根"),
            ("乙", "NNLL", "N区第一键", "折区起首字根"),
            ("九", "VTNG", "V区第二键", "字根：乙"),
            ("力", "LTNN", "L键", "字根：力"),
            ("乃", "DETN", "N键", "字根：乃"),
            ("刀", "VNTE", "V键", "字根：刀"),
            ("卜", "HHYD", "H键", "字根：卜"),
        ]
        .iter()
        .map(|(c, code, pos, desc)| doc! { "character": *c, "code": *code, "position": *pos, "description": *desc })
        .collect();
        roots_col.insert_many(roots_docs, None).await.map_err(|e| format!("Failed to insert roots: {}", e))?;

        // --- key_radicals ---
        let key_radicals_col = db.collection::<Document>("key_radicals");
        key_radicals_col.delete_many(doc! {}, None).await.ok();
        let key_radicals_docs: Vec<Document> = [
            ("g", "王、一、五、戋", "王旁青头戋（兼）五一", "G区横区第一键，包含横笔和戈字根"),
            ("f", "土、士、二、干、十、寸、雨", "土士二干十寸雨", "F区横区第二键，包含土字根"),
            ("d", "大、犬、三、古、石、厂", "大犬三（古）石厂", "D区横区第三键，包含大字根"),
            ("s", "木、丁、西", "木丁西", "S区横区第四键，包含木字根"),
            ("a", "工、戈、艹、七、廿", "工戈草头右框七", "A区横区第五键，包含工字根"),
            ("h", "目、止、卜、虍、上", "目具上止卜虎皮", "H区竖区第一键，包含目字根"),
            ("j", "日、早、虫、刂、竖", "日早两竖与虫依", "J区竖区第二键，包含日字根"),
            ("k", "口、川", "口与川，字根稀", "K区竖区第三键，包含口字根"),
            ("l", "田、甲、四、车、囗", "田甲方框四车里", "L区竖区第四键，包含田字根"),
            ("m", "山、由、贝、几", "山由贝，下框几", "M区竖区第五键，包含山字根"),
            ("t", "禾、竹、丿、彳、攵", "禾竹一撇双人立", "T区撇区第一键，包含禾字根"),
            ("r", "白、手、斤、牛", "白手看头三二斤", "R区撇区第二键，包含白字根"),
            ("e", "舟、用、月、豕、衣", "舟用乃月豕（家）衣", "E区撇区第三键，包含月字根"),
            ("w", "人、八、亻", "人八登头单人几", "W区撇区第四键，包含人字根"),
            ("q", "金、饣、勹、儿、夕", "金勺缺点无尾鱼，犬旁留叉", "Q区撇区第五键，包含金字根"),
            ("y", "言、文、方、广、丶", "言文方广在四一，高头一捺谁人去", "Y区捺区第一键，包含言字根"),
            ("u", "立、辛、六、门、疒", "立辛两点六门疒（病）", "U区捺区第二键，包含立字根"),
            ("i", "氵（三点水）、小", "水旁兴头小倒立", "I区捺区第三键，包含水字根"),
            ("o", "火、米、灬", "火业头，四点米", "O区捺区第四键，包含火字根"),
            ("p", "之、宀（宝盖）、冖、礻、衤", "之字军盖建道底，摘礻衤", "P区捺区第五键，包含之字根"),
            ("n", "已、己、巳、尸、心、羽", "已半巳满不出己，左框折尸心和羽", "N区折区第一键，包含已字根"),
            ("b", "子、耳、了、也、卩", "子耳了也框向上", "B区折区第二键，包含子字根"),
            ("v", "女、刀、九、臼", "女刀九臼山朝西", "V区折区第三键，包含女字根"),
            ("c", "又、巴、马、厶", "又巴马，丢矢矣", "C区折区第四键，包含又字根"),
            ("x", "幺、母、弓、匕", "慈母无心弓和匕，幼无力", "X区折区第五键，包含丝字根"),
        ]
        .iter()
        .map(|(k, r, f, d)| doc! { "key_char": *k, "radicals": *r, "formula": *f, "description": *d })
        .collect();
        key_radicals_col.insert_many(key_radicals_docs, None).await.map_err(|e| format!("Failed to insert key_radicals: {}", e))?;

        // --- english_texts ---
        let english_texts_col = db.collection::<Document>("english_texts");
        english_texts_col.delete_many(doc! {}, None).await.ok();
        let english_texts_docs: Vec<Document> = [
            ("基础练习", "the quick brown fox jumps over the lazy dog", "easy"),
            ("常用句子", "hello world this is a typing practice text for english learning", "easy"),
            ("进阶练习", "practice makes perfect keep typing to improve your speed and accuracy", "medium"),
            ("高级练习", "the five boxing wizards jump quickly at dawn every single day", "hard"),
        ]
        .iter()
        .map(|(t, c, d)| doc! { "title": *t, "content": *c, "difficulty": *d })
        .collect();
        english_texts_col.insert_many(english_texts_docs, None).await.map_err(|e| format!("Failed to insert english_texts: {}", e))?;

        // --- japanese_keyboards ---
        let japanese_keyboards_col = db.collection::<Document>("japanese_keyboards");
        japanese_keyboards_col.delete_many(doc! {}, None).await.ok();
        japanese_keyboards_col
            .insert_one(doc! { "layout_name": "QWERTY Japanese", "description": "标准QWERTY键盘的日语布局，使用罗马字输入" }, None)
            .await
            .map_err(|e| format!("Failed to insert japanese_keyboards: {}", e))?;

        // --- japanese_characters ---
        let japanese_characters_col = db.collection::<Document>("japanese_characters");
        japanese_characters_col.delete_many(doc! {}, None).await.ok();
        let japanese_characters_docs: Vec<Document> = [
            ("あ", "a", "a", 1), ("い", "i", "i", 1), ("う", "u", "u", 1),
            ("え", "e", "e", 1), ("お", "o", "o", 1), ("か", "ka", "k", 1),
            ("き", "ki", "k", 1), ("く", "ku", "k", 1), ("け", "ke", "k", 1),
            ("こ", "ko", "k", 1), ("さ", "sa", "s", 1), ("し", "shi", "s", 1),
            ("す", "su", "s", 1), ("せ", "se", "s", 1), ("そ", "so", "s", 1),
        ]
        .iter()
        .map(|(c, r, k, l)| doc! { "character": *c, "romanji": *r, "keyboard_key": *k, "layout_id": *l })
        .collect();
        japanese_characters_col.insert_many(japanese_characters_docs, None).await.map_err(|e| format!("Failed to insert japanese_characters: {}", e))?;

        // --- japanese_texts ---
        let japanese_texts_col = db.collection::<Document>("japanese_texts");
        japanese_texts_col.delete_many(doc! {}, None).await.ok();
        let japanese_texts_docs: Vec<Document> = [
            ("平假名基础", "あいうえおかきくけこさしすせそ", "easy", "hiragana"),
            ("片假名基础", "アイウエオカキクケコサシスセソ", "easy", "katakana"),
            ("汉字基础", "一二三四五六七八九十", "easy", "kanji"),
            ("混合练习", "こんにちは、元気ですか？私は元気です。", "easy", "mixed"),
            ("平假名进阶", "いろはにほへとちりぬるをわがよたれぞつねならむ", "medium", "hiragana"),
            ("片假名进阶", "イロハニホヘトチリヌルヲワガヨタレゾツネナラム", "medium", "katakana"),
            ("汉字进阶", "日本語の勉強は楽しいです。毎日勉強しています。", "medium", "kanji"),
            ("混合进阶", "東京は日本の首都です。人口が多くてにぎやかです。", "medium", "mixed"),
            ("高级平假名", "むかしむかしあるところにおじいさんとおばあさんがいました", "hard", "hiragana"),
            ("高级片假名", "ムカシムカシアルトコロニオジイサントオバアサンガイマシタ", "hard", "katakana"),
            ("高级汉字", "日本国は、東アジアの島国であり、首都は東京です。", "hard", "kanji"),
            ("高级混合", "今日はいい天気ですね。公園に行きませんか？", "hard", "mixed"),
        ]
        .iter()
        .map(|(t, c, d, ty)| doc! { "title": *t, "content": *c, "difficulty": *d, "type": *ty })
        .collect();
        japanese_texts_col.insert_many(japanese_texts_docs, None).await.map_err(|e| format!("Failed to insert japanese_texts: {}", e))?;

        // --- bopomofo_keyboards ---
        let bopomofo_keyboards_col = db.collection::<Document>("bopomofo_keyboards");
        bopomofo_keyboards_col.delete_many(doc! {}, None).await.ok();
        bopomofo_keyboards_col
            .insert_one(doc! { "layout_name": "Bopomofo Keyboard", "description": "标准注音键盘布局，使用注音符号输入" }, None)
            .await
            .map_err(|e| format!("Failed to insert bopomofo_keyboards: {}", e))?;

        // --- bopomofo_characters ---
        let bopomofo_characters_col = db.collection::<Document>("bopomofo_characters");
        bopomofo_characters_col.delete_many(doc! {}, None).await.ok();
        let bopomofo_characters_docs: Vec<Document> = [
            ("ㄅ", "b", "b", 1), ("ㄆ", "p", "p", 1), ("ㄇ", "m", "m", 1),
            ("ㄈ", "f", "f", 1), ("ㄉ", "d", "d", 1), ("ㄊ", "t", "t", 1),
            ("ㄋ", "n", "n", 1), ("ㄌ", "l", "l", 1), ("ㄍ", "g", "g", 1),
            ("ㄎ", "k", "k", 1), ("ㄏ", "h", "h", 1), ("ㄐ", "j", "j", 1),
            ("ㄑ", "q", "q", 1), ("ㄒ", "x", "x", 1), ("ㄓ", "zh", "z", 1),
            ("ㄔ", "ch", "c", 1), ("ㄕ", "sh", "s", 1), ("ㄖ", "r", "r", 1),
            ("ㄗ", "z", "z", 1), ("ㄘ", "c", "c", 1), ("ㄙ", "s", "s", 1),
            ("ㄧ", "i", "i", 1), ("ㄨ", "u", "u", 1), ("ㄩ", "ü", "v", 1),
            ("ㄚ", "a", "a", 1), ("ㄛ", "o", "o", 1), ("ㄜ", "e", "e", 1),
            ("ㄝ", "ê", "e", 1), ("ㄞ", "ai", "a", 1), ("ㄟ", "ei", "e", 1),
            ("ㄠ", "ao", "a", 1), ("ㄡ", "ou", "o", 1), ("ㄢ", "an", "a", 1),
            ("ㄣ", "en", "e", 1), ("ㄤ", "ang", "a", 1), ("ㄥ", "eng", "e", 1),
            ("ㄦ", "er", "e", 1),
        ]
        .iter()
        .map(|(c, b, k, l)| doc! { "character": *c, "bopomofo": *b, "keyboard_key": *k, "layout_id": *l })
        .collect();
        bopomofo_characters_col.insert_many(bopomofo_characters_docs, None).await.map_err(|e| format!("Failed to insert bopomofo_characters: {}", e))?;

        // --- traditional_chinese_texts ---
        let traditional_chinese_texts_col = db.collection::<Document>("traditional_chinese_texts");
        traditional_chinese_texts_col.delete_many(doc! {}, None).await.ok();
        let traditional_chinese_texts_docs: Vec<Document> = [
            ("基础练习", "一二人三四五六七八九十", "easy"),
            ("常用句子", "你好，歡迎光臨！請問有什麼可以幫您的嗎？", "easy"),
            ("进阶练习", "中華文化源遠流長，博大精深，值得我們好好學習。", "medium"),
            ("高级练习", "臺灣是一個美麗的島嶼，擁有豐富的自然資源和人文景觀。", "hard"),
        ]
        .iter()
        .map(|(t, c, d)| doc! { "title": *t, "content": *c, "difficulty": *d })
        .collect();
        traditional_chinese_texts_col.insert_many(traditional_chinese_texts_docs, None).await.map_err(|e| format!("Failed to insert traditional_chinese_texts: {}", e))?;

        // --- japanese_gojuon ---
        let japanese_gojuon_col = db.collection::<Document>("japanese_gojuon");
        japanese_gojuon_col.delete_many(doc! {}, None).await.ok();
        let japanese_gojuon_docs: Vec<Document> = [
            ("あ", "a", "あ行", "あ段", "ア"), ("い", "i", "あ行", "い段", "イ"),
            ("う", "u", "あ行", "う段", "ウ"), ("え", "e", "あ行", "え段", "エ"),
            ("お", "o", "あ行", "お段", "オ"), ("か", "ka", "か行", "あ段", "カ"),
            ("き", "ki", "か行", "い段", "キ"), ("く", "ku", "か行", "う段", "ク"),
            ("け", "ke", "か行", "え段", "ケ"), ("こ", "ko", "か行", "お段", "コ"),
            ("さ", "sa", "さ行", "あ段", "サ"), ("し", "shi", "さ行", "い段", "シ"),
            ("す", "su", "さ行", "う段", "ス"), ("せ", "se", "さ行", "え段", "セ"),
            ("そ", "so", "さ行", "お段", "ソ"), ("た", "ta", "た行", "あ段", "タ"),
            ("ち", "chi", "た行", "い段", "チ"), ("つ", "tsu", "た行", "う段", "ツ"),
            ("て", "te", "た行", "え段", "テ"), ("と", "to", "た行", "お段", "ト"),
            ("な", "na", "な行", "あ段", "ナ"), ("に", "ni", "な行", "い段", "ニ"),
            ("ぬ", "nu", "な行", "う段", "ヌ"), ("ね", "ne", "な行", "え段", "ネ"),
            ("の", "no", "な行", "お段", "ノ"), ("は", "ha", "は行", "あ段", "ハ"),
            ("ひ", "hi", "は行", "い段", "ヒ"), ("ふ", "fu", "は行", "う段", "フ"),
            ("へ", "he", "は行", "え段", "ヘ"), ("ほ", "ho", "は行", "お段", "ホ"),
            ("ま", "ma", "ま行", "あ段", "マ"), ("み", "mi", "ま行", "い段", "ミ"),
            ("む", "mu", "ま行", "う段", "ム"), ("め", "me", "ま行", "え段", "メ"),
            ("も", "mo", "ま行", "お段", "モ"), ("や", "ya", "や行", "あ段", "ヤ"),
            ("ゆ", "yu", "や行", "う段", "ユ"), ("よ", "yo", "や行", "お段", "ヨ"),
            ("ら", "ra", "ら行", "あ段", "ラ"), ("り", "ri", "ら行", "い段", "リ"),
            ("る", "ru", "ら行", "う段", "ル"), ("れ", "re", "ら行", "え段", "レ"),
            ("ろ", "ro", "ら行", "お段", "ロ"), ("わ", "wa", "わ行", "あ段", "ワ"),
            ("を", "wo", "わ行", "お段", "ヲ"), ("ん", "n", "ん行", "ん段", "ン"),
        ]
        .iter()
        .map(|(c, r, row, col, p)| doc! { "character": *c, "romanji": *r, "row": *row, "column": *col, "pronunciation": *p })
        .collect();
        japanese_gojuon_col.insert_many(japanese_gojuon_docs, None).await.map_err(|e| format!("Failed to insert japanese_gojuon: {}", e))?;

        // --- japanese_kanji ---
        let japanese_kanji_col = db.collection::<Document>("japanese_kanji");
        japanese_kanji_col.delete_many(doc! {}, None).await.ok();
        let japanese_kanji_docs: Vec<Document> = [
            ("一", "いち", "一", "N5"), ("二", "に", "二", "N5"), ("三", "さん", "三", "N5"),
            ("四", "よん", "四", "N5"), ("五", "ご", "五", "N5"), ("六", "ろく", "六", "N5"),
            ("七", "なな", "七", "N5"), ("八", "はち", "八", "N5"), ("九", "きゅう", "九", "N5"),
            ("十", "じゅう", "十", "N5"), ("日", "にち", "日", "N5"), ("月", "つき", "月", "N5"),
            ("火", "ひ", "火", "N5"), ("水", "みず", "水", "N5"), ("木", "き", "木", "N5"),
            ("金", "かね", "金", "N5"), ("土", "つち", "土", "N5"), ("人", "ひと", "人", "N5"),
            ("大", "おお", "大", "N5"), ("小", "ちい", "小", "N5"),
        ]
        .iter()
        .map(|(k, r, m, l)| doc! { "kanji": *k, "reading": *r, "meaning": *m, "level": *l })
        .collect();
        japanese_kanji_col.insert_many(japanese_kanji_docs, None).await.map_err(|e| format!("Failed to insert japanese_kanji: {}", e))?;

        println!("All sample data imported to MongoDB");
        Ok(())
    }

    async fn next_id(&self, collection_name: &str) -> Result<i32, String> {
        let db = self.get_db();
        let counter_col = db.collection::<Document>("_counters");
        let options = FindOneAndUpdateOptions::builder()
            .upsert(true)
            .return_document(ReturnDocument::After)
            .build();
        let result = counter_col
            .find_one_and_update(
                doc! { "_id": collection_name },
                doc! { "$inc": { "seq": 1 } },
                options,
            )
            .await
            .map_err(|e| format!("Failed to generate id: {}", e))?;
        let id = result.and_then(|d| d.get_i32("seq").ok()).unwrap_or(1);
        Ok(id)
    }
}

#[async_trait]
impl Database for MongoDatabase {
    async fn init_db(&self) -> Result<(), String> {
        let db = self.get_db();
        let collections = db
            .list_collection_names(None)
            .await
            .map_err(|e| format!("Failed to list collections: {}", e))?;

        let needs_import = if !collections.contains(&"wubi_characters".to_string()) {
            true
        } else {
            let chars_collection = db.collection::<Document>("wubi_characters");
            let count = chars_collection
                .count_documents(doc! {}, None)
                .await
                .map_err(|e| format!("Failed to count wubi characters: {}", e))?;
            // Re-import if collection has fewer than expected entries (136916)
            count < 100000
        };

        if needs_import {
            self.import_wubi_dict().await?;
            self.import_sample_data().await?;
        } else {
            println!("MongoDB data already initialized, skipping import.");
        }
        Ok(())
    }

    async fn get_wubi_code(&self, character: &str) -> Result<WubiCharacter, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("wubi_characters");
        let doc = collection
            .find_one(doc! { "character": character }, None)
            .await
            .map_err(|e| format!("Failed to query wubi code: {}", e))?;
        match doc {
            Some(doc) => {
                let id = doc.get_i32("id").unwrap_or(0);
                let character = doc
                    .get_str("character")
                    .map_err(|e| e.to_string())?
                    .to_string();
                let simple_code = doc.get_str("simple_code").map_err(|_| "")?.to_string();
                let full_code = doc.get_str("full_code").map_err(|_| "")?.to_string();
                let pinyin = doc.get_str("pinyin").map_err(|_| "")?.to_string();
                let remark = doc.get_str("remark").map_err(|_| "")?.to_string();
                Ok(WubiCharacter {
                    id,
                    character,
                    simple_code,
                    full_code,
                    pinyin,
                    remark,
                })
            }
            None => Err("Character not found".to_string()),
        }
    }

    async fn get_all_wubi_characters(&self) -> Result<Vec<WubiCharacter>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("wubi_characters");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to query characters: {}", e))?;
        let mut characters = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e: mongodb::error::Error| e.to_string())?
        {
            let id = doc.get_i32("id").unwrap_or(0);
            let character = doc
                .get_str("character")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let simple_code = doc.get_str("simple_code").map_err(|_| "")?.to_string();
            let full_code = doc.get_str("full_code").map_err(|_| "")?.to_string();
            let pinyin = doc.get_str("pinyin").map_err(|_| "")?.to_string();
            let remark = doc.get_str("remark").map_err(|_| "")?.to_string();
            characters.push(WubiCharacter {
                id,
                character,
                simple_code,
                full_code,
                pinyin,
                remark,
            });
        }
        characters.sort_by(|a, b| a.character.cmp(&b.character));
        Ok(characters)
    }

    async fn get_lessons(&self) -> Result<Vec<Lesson>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("lessons");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e: mongodb::error::Error| format!("Failed to query lessons: {}", e))?;
        let mut lessons = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e: mongodb::error::Error| e.to_string())?
        {
            let id = doc
                .get_object_id("_id")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
            let character = doc
                .get_str("character")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let code = doc
                .get_str("code")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let description = doc
                .get_str("description")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            lessons.push(Lesson {
                id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(),
                character,
                code,
                description,
            });
        }
        lessons.sort_by_key(|l| l.id);
        Ok(lessons)
    }

    async fn get_lesson_by_id(&self, id: i32) -> Result<Lesson, String> {
        let lessons = self.get_lessons().await?;
        lessons
            .into_iter()
            .find(|l| l.id == id)
            .ok_or_else(|| "Lesson not found".to_string())
    }

    async fn create_lesson(
        &self,
        character: &str,
        code: &str,
        description: &str,
    ) -> Result<Lesson, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("lessons");
        let doc = doc! { "character": character, "code": code, "description": description };
        let result = collection
            .insert_one(doc, None)
            .await
            .map_err(|e| format!("Failed to create lesson: {}", e))?;
        let id = result
            .inserted_id
            .as_object_id()
            .map(|oid| oid.to_hex().chars().take(8).map(|c| c as i32).sum())
            .unwrap_or(0);
        Ok(Lesson {
            id,
            character: character.to_string(),
            code: code.to_string(),
            description: description.to_string(),
        })
    }

    async fn get_articles(&self) -> Result<Vec<Article>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("articles");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e: mongodb::error::Error| format!("Failed to query articles: {}", e))?;
        let mut articles = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e: mongodb::error::Error| e.to_string())?
        {
            let id = doc
                .get_object_id("_id")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
            let title = doc
                .get_str("title")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let content = doc
                .get_str("content")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let difficulty = doc
                .get_str("difficulty")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            articles.push(Article {
                id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(),
                title,
                content,
                difficulty,
            });
        }
        articles.sort_by_key(|a| a.id);
        Ok(articles)
    }

    async fn get_article_by_id(&self, id: i32) -> Result<Article, String> {
        let articles = self.get_articles().await?;
        articles
            .into_iter()
            .find(|a| a.id == id)
            .ok_or_else(|| "Article not found".to_string())
    }

    async fn create_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("articles");
        let doc = doc! { "title": title, "content": content, "difficulty": difficulty };
        let result = collection
            .insert_one(doc, None)
            .await
            .map_err(|e| format!("Failed to create article: {}", e))?;
        let id = result
            .inserted_id
            .as_object_id()
            .map(|oid| oid.to_hex().chars().take(8).map(|c| c as i32).sum())
            .unwrap_or(0);
        Ok(Article {
            id,
            title: title.to_string(),
            content: content.to_string(),
            difficulty: difficulty.to_string(),
        })
    }

    async fn update_article(
        &self,
        id: i32,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("articles");
        collection
            .update_many(
                doc! { "id": id },
                doc! { "$set": { "title": title, "content": content, "difficulty": difficulty } },
                None,
            )
            .await
            .map_err(|e| format!("Failed to update article: {}", e))?;
        Ok(Article {
            id,
            title: title.to_string(),
            content: content.to_string(),
            difficulty: difficulty.to_string(),
        })
    }

    async fn delete_article(&self, id: i32) -> Result<(), String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("articles");
        collection
            .delete_many(doc! { "id": id }, None)
            .await
            .map_err(|e| format!("Failed to delete article: {}", e))?;
        Ok(())
    }

    async fn get_custom_articles(&self) -> Result<Vec<Article>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("custom_articles");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e: mongodb::error::Error| format!("Failed to query custom articles: {}", e))?;
        let mut articles = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e: mongodb::error::Error| e.to_string())?
        {
            let id = doc
                .get_object_id("_id")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
            let title = doc
                .get_str("title")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let content = doc
                .get_str("content")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let difficulty = doc
                .get_str("difficulty")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            articles.push(Article {
                id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(),
                title,
                content,
                difficulty,
            });
        }
        articles.sort_by_key(|a| a.id);
        Ok(articles)
    }

    async fn create_custom_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("custom_articles");
        let doc = doc! { "title": title, "content": content, "difficulty": difficulty };
        let result = collection
            .insert_one(doc, None)
            .await
            .map_err(|e| format!("Failed to create custom article: {}", e))?;
        let id = result
            .inserted_id
            .as_object_id()
            .map(|oid| oid.to_hex().chars().take(8).map(|c| c as i32).sum())
            .unwrap_or(0);
        Ok(Article {
            id,
            title: title.to_string(),
            content: content.to_string(),
            difficulty: difficulty.to_string(),
        })
    }

    async fn delete_custom_article(&self, id: i32) -> Result<(), String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("custom_articles");
        collection
            .delete_many(doc! { "id": id }, None)
            .await
            .map_err(|e| format!("Failed to delete custom article: {}", e))?;
        Ok(())
    }

    async fn update_wubi_code(
        &self,
        character: &str,
        new_code: &str,
    ) -> Result<WubiCharacter, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("wubi_characters");
        let filter = doc! { "character": character };
        let update = doc! { "$set": { "full_code": new_code } };
        let opts = mongodb::options::UpdateOptions::builder()
            .upsert(true)
            .build();
        collection
            .update_one(filter, update, opts)
            .await
            .map_err(|e| format!("Failed to update wubi code: {}", e))?;
        Ok(WubiCharacter {
            id: 0,
            character: character.to_string(),
            simple_code: String::new(),
            full_code: new_code.to_string(),
            pinyin: String::new(),
            remark: String::new(),
        })
    }

    async fn get_wubi_roots(&self) -> Result<Vec<WubiRoot>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("wubi_roots");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e: mongodb::error::Error| format!("Failed to query roots: {}", e))?;
        let mut roots = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e: mongodb::error::Error| e.to_string())?
        {
            let id = doc
                .get_object_id("_id")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
            let character = doc
                .get_str("character")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let code = doc
                .get_str("code")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let position = doc
                .get_str("position")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let description = doc
                .get_str("description")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            roots.push(WubiRoot {
                id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(),
                character,
                code,
                position,
                description,
            });
        }
        roots.sort_by_key(|r| r.position.clone());
        Ok(roots)
    }

    async fn get_wubi_root_by_id(&self, id: i32) -> Result<WubiRoot, String> {
        let roots = self.get_wubi_roots().await?;
        roots
            .into_iter()
            .find(|r| r.id == id)
            .ok_or_else(|| "Wubi root not found".to_string())
    }

    async fn create_wubi_root(
        &self,
        character: &str,
        code: &str,
        position: &str,
        description: &str,
    ) -> Result<WubiRoot, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("wubi_roots");
        let doc = doc! { "character": character, "code": code, "position": position, "description": description };
        let result = collection
            .insert_one(doc, None)
            .await
            .map_err(|e| format!("Failed to create root: {}", e))?;
        let id = result
            .inserted_id
            .as_object_id()
            .map(|oid| oid.to_hex().chars().take(8).map(|c| c as i32).sum())
            .unwrap_or(0);
        Ok(WubiRoot {
            id,
            character: character.to_string(),
            code: code.to_string(),
            position: position.to_string(),
            description: description.to_string(),
        })
    }

    async fn search_wubi_root(&self, character: &str) -> Result<Option<WubiRoot>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("wubi_roots");
        let doc = collection
            .find_one(doc! { "character": character }, None)
            .await
            .map_err(|e: mongodb::error::Error| format!("Failed to query root: {}", e))?;
        match doc {
            Some(doc) => {
                let id = doc
                    .get_object_id("_id")
                    .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
                let character = doc
                    .get_str("character")
                    .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                    .to_string();
                let code = doc
                    .get_str("code")
                    .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                    .to_string();
                let position = doc
                    .get_str("position")
                    .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                    .to_string();
                let description = doc
                    .get_str("description")
                    .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                    .to_string();
                Ok(Some(WubiRoot {
                    id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(),
                    character,
                    code,
                    position,
                    description,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("users");
        let doc = collection
            .find_one(doc! { "username": username }, None)
            .await
            .map_err(|e: mongodb::error::Error| format!("Failed to query user: {}", e))?;
        match doc {
            Some(doc) => {
                let id = doc
                    .get_object_id("_id")
                    .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
                let username = doc
                    .get_str("username")
                    .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                    .to_string();
                let email = doc
                    .get_str("email")
                    .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                    .to_string();
                let password_hash = doc
                    .get_str("password_hash")
                    .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                    .to_string();
                let created_at = doc
                    .get_str("created_at")
                    .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                    .to_string();
                Ok(Some(User {
                    id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(),
                    username,
                    email,
                    password_hash,
                    created_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
        let users = self.get_all_users().await?;
        Ok(users.into_iter().find(|u| u.id == id))
    }

    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("users");
        let now = chrono::Utc::now().to_rfc3339();
        let doc = doc! { "username": username, "email": email, "password_hash": password_hash, "created_at": now.clone() };
        let result = collection
            .insert_one(doc, None)
            .await
            .map_err(|e| format!("Failed to create user: {}", e))?;
        let id = result
            .inserted_id
            .as_object_id()
            .map(|oid| oid.to_hex().chars().take(8).map(|c| c as i32).sum())
            .unwrap_or(0);
        Ok(User {
            id,
            username: username.to_string(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            created_at: now,
        })
    }

    async fn save_progress(
        &self,
        user_name: &str,
        lesson_id: i32,
        accuracy: f32,
        score: i32,
    ) -> Result<(), String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("user_progress");
        let now = chrono::Utc::now().to_rfc3339();
        let doc = doc! { "user_name": user_name, "lesson_id": lesson_id, "accuracy": accuracy, "score": score, "updated_at": now };
        collection
            .insert_one(doc, None)
            .await
            .map_err(|e| format!("Failed to save progress: {}", e))?;
        Ok(())
    }

    async fn get_key_radicals(&self) -> Result<Vec<KeyRadical>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("key_radicals");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to query key_radicals: {}", e))?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let id = doc.get_i32("id").unwrap_or(0);
            let key_char = doc.get_str("key_char").unwrap_or("").to_string();
            let radicals = doc.get_str("radicals").unwrap_or("").to_string();
            let formula = doc.get_str("formula").unwrap_or("").to_string();
            let description = doc.get_str("description").unwrap_or("").to_string();
            results.push(KeyRadical {
                id,
                key_char,
                radicals,
                formula,
                description,
            });
        }
        Ok(results)
    }

    async fn get_key_radical_by_key(&self, key_char: &str) -> Result<Option<KeyRadical>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("key_radicals");
        let doc = collection
            .find_one(doc! { "key_char": key_char }, None)
            .await
            .map_err(|e| format!("Failed to query key_radical: {}", e))?;
        match doc {
            Some(doc) => {
                let id = doc.get_i32("id").unwrap_or(0);
                let key_char = doc.get_str("key_char").unwrap_or("").to_string();
                let radicals = doc.get_str("radicals").unwrap_or("").to_string();
                let formula = doc.get_str("formula").unwrap_or("").to_string();
                let description = doc.get_str("description").unwrap_or("").to_string();
                Ok(Some(KeyRadical {
                    id,
                    key_char,
                    radicals,
                    formula,
                    description,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_english_texts(&self) -> Result<Vec<EnglishText>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("english_texts");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to query english_texts: {}", e))?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let id = doc.get_i32("id").unwrap_or(0);
            let title = doc.get_str("title").unwrap_or("").to_string();
            let content = doc.get_str("content").unwrap_or("").to_string();
            let difficulty = doc.get_str("difficulty").unwrap_or("").to_string();
            results.push(EnglishText {
                id,
                title,
                content,
                difficulty,
            });
        }
        Ok(results)
    }

    async fn get_japanese_texts(&self) -> Result<Vec<JapaneseText>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("japanese_texts");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to query japanese_texts: {}", e))?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let id = doc.get_i32("id").unwrap_or(0);
            let title = doc.get_str("title").unwrap_or("").to_string();
            let content = doc.get_str("content").unwrap_or("").to_string();
            let difficulty = doc.get_str("difficulty").unwrap_or("").to_string();
            let r#type = doc.get_str("type").unwrap_or("mixed").to_string();
            results.push(JapaneseText {
                id,
                title,
                content,
                difficulty,
                r#type,
            });
        }
        Ok(results)
    }

    async fn get_japanese_keyboards(&self) -> Result<Vec<JapaneseKeyboard>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("japanese_keyboards");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to query japanese_keyboards: {}", e))?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let id = doc.get_i32("id").unwrap_or(0);
            let layout_name = doc.get_str("layout_name").unwrap_or("").to_string();
            let description = doc.get_str("description").unwrap_or("").to_string();
            results.push(JapaneseKeyboard {
                id,
                layout_name,
                description,
            });
        }
        Ok(results)
    }

    async fn get_japanese_characters(&self) -> Result<Vec<JapaneseCharacter>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("japanese_characters");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to query japanese_characters: {}", e))?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let id = doc.get_i32("id").unwrap_or(0);
            let character = doc.get_str("character").unwrap_or("").to_string();
            let romanji = doc.get_str("romanji").unwrap_or("").to_string();
            let keyboard_key = doc.get_str("keyboard_key").unwrap_or("").to_string();
            let layout_id = doc.get_i32("layout_id").unwrap_or(0);
            results.push(JapaneseCharacter {
                id,
                character,
                romanji,
                keyboard_key,
                layout_id,
            });
        }
        Ok(results)
    }

    async fn get_traditional_chinese_texts(&self) -> Result<Vec<TraditionalChineseText>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("traditional_chinese_texts");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to query traditional_chinese_texts: {}", e))?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let id = doc.get_i32("id").unwrap_or(0);
            let title = doc.get_str("title").unwrap_or("").to_string();
            let content = doc.get_str("content").unwrap_or("").to_string();
            let difficulty = doc.get_str("difficulty").unwrap_or("").to_string();
            results.push(TraditionalChineseText {
                id,
                title,
                content,
                difficulty,
            });
        }
        Ok(results)
    }

    async fn get_bopomofo_keyboards(&self) -> Result<Vec<BopomofoKeyboard>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("bopomofo_keyboards");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to query bopomofo_keyboards: {}", e))?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let id = doc.get_i32("id").unwrap_or(0);
            let layout_name = doc.get_str("layout_name").unwrap_or("").to_string();
            let description = doc.get_str("description").unwrap_or("").to_string();
            results.push(BopomofoKeyboard {
                id,
                layout_name,
                description,
            });
        }
        Ok(results)
    }

    async fn get_bopomofo_characters(&self) -> Result<Vec<BopomofoCharacter>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("bopomofo_characters");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to query bopomofo_characters: {}", e))?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let id = doc.get_i32("id").unwrap_or(0);
            let character = doc.get_str("character").unwrap_or("").to_string();
            let bopomofo = doc.get_str("bopomofo").unwrap_or("").to_string();
            let keyboard_key = doc.get_str("keyboard_key").unwrap_or("").to_string();
            let layout_id = doc.get_i32("layout_id").unwrap_or(0);
            results.push(BopomofoCharacter {
                id,
                character,
                bopomofo,
                keyboard_key,
                layout_id,
            });
        }
        Ok(results)
    }

    async fn get_japanese_gojuon(&self) -> Result<Vec<JapaneseGojuon>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("japanese_gojuon");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to query japanese_gojuon: {}", e))?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let id = doc.get_i32("id").unwrap_or(0);
            let character = doc.get_str("character").unwrap_or("").to_string();
            let romanji = doc.get_str("romanji").unwrap_or("").to_string();
            let row = doc.get_str("row").unwrap_or("").to_string();
            let column = doc.get_str("column").unwrap_or("").to_string();
            let pronunciation = doc.get_str("pronunciation").unwrap_or("").to_string();
            results.push(JapaneseGojuon {
                id,
                character,
                romanji,
                row,
                column,
                pronunciation,
            });
        }
        Ok(results)
    }

    async fn get_japanese_kanji(&self) -> Result<Vec<JapaneseKanji>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("japanese_kanji");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e| format!("Failed to query japanese_kanji: {}", e))?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await.map_err(|e| e.to_string())? {
            let id = doc.get_i32("id").unwrap_or(0);
            let kanji = doc.get_str("kanji").unwrap_or("").to_string();
            let reading = doc.get_str("reading").unwrap_or("").to_string();
            let meaning = doc.get_str("meaning").unwrap_or("").to_string();
            let level = doc.get_str("level").unwrap_or("").to_string();
            results.push(JapaneseKanji {
                id,
                kanji,
                reading,
                meaning,
                level,
            });
        }
        Ok(results)
    }

    async fn get_japanese_kanji_by_char(
        &self,
        kanji: &str,
    ) -> Result<Option<JapaneseKanji>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("japanese_kanji");
        let doc = collection
            .find_one(doc! { "kanji": kanji }, None)
            .await
            .map_err(|e| format!("Failed to query japanese_kanji: {}", e))?;

        match doc {
            Some(doc) => {
                let id = doc.get_i32("id").unwrap_or(0);
                let kanji = doc.get_str("kanji").unwrap_or("").to_string();
                let reading = doc.get_str("reading").unwrap_or("").to_string();
                let meaning = doc.get_str("meaning").unwrap_or("").to_string();
                let level = doc.get_str("level").unwrap_or("").to_string();
                Ok(Some(JapaneseKanji {
                    id,
                    kanji,
                    reading,
                    meaning,
                    level,
                }))
            }
            None => Ok(None),
        }
    }
}

impl MongoDatabase {
    async fn get_all_users(&self) -> Result<Vec<User>, String> {
        let db = self.get_db();
        let collection = db.collection::<Document>("users");
        let mut cursor = collection
            .find(doc! {}, None)
            .await
            .map_err(|e: mongodb::error::Error| format!("Failed to query users: {}", e))?;
        let mut users = Vec::new();
        while let Some(doc) = cursor
            .try_next()
            .await
            .map_err(|e: mongodb::error::Error| e.to_string())?
        {
            let id = doc
                .get_object_id("_id")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?;
            let username = doc
                .get_str("username")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let email = doc
                .get_str("email")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let password_hash = doc
                .get_str("password_hash")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            let created_at = doc
                .get_str("created_at")
                .map_err(|e: mongodb::bson::document::ValueAccessError| e.to_string())?
                .to_string();
            users.push(User {
                id: id.to_hex().chars().take(8).map(|c: char| c as i32).sum(),
                username,
                email,
                password_hash,
                created_at,
            });
        }
        Ok(users)
    }
}

/// PostgreSQL数据库实现
pub struct PostgresDatabase {
    pool: Pool<Postgres>,
}

impl PostgresDatabase {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Database for PostgresDatabase {
    async fn init_db(&self) -> Result<(), String> {
        // 创建表结构
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id SERIAL PRIMARY KEY,
                username VARCHAR(64) NOT NULL UNIQUE,
                email VARCHAR(255) NOT NULL UNIQUE,
                password_hash VARCHAR(255) NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS lessons (
                id SERIAL PRIMARY KEY,
                character_val VARCHAR(32) NOT NULL,
                code VARCHAR(32) NOT NULL,
                description TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS articles (
                id SERIAL PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium'
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS custom_articles (
                id SERIAL PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'custom'
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wubi_characters (
                id SERIAL PRIMARY KEY,
                character_val VARCHAR(64) NOT NULL UNIQUE,
                simple_code VARCHAR(8) NOT NULL DEFAULT '',
                full_code VARCHAR(8) NOT NULL DEFAULT '',
                pinyin VARCHAR(64) NOT NULL DEFAULT '',
                remark VARCHAR(128) NOT NULL DEFAULT '',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS english_texts (
                id SERIAL PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium'
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS japanese_keyboards (
                id SERIAL PRIMARY KEY,
                layout_name VARCHAR(100) NOT NULL UNIQUE,
                description TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS japanese_characters (
                id SERIAL PRIMARY KEY,
                "character" VARCHAR(10) NOT NULL,
                romanji VARCHAR(50) NOT NULL,
                keyboard_key VARCHAR(10) NOT NULL,
                layout_id INT NOT NULL,
                FOREIGN KEY (layout_id) REFERENCES japanese_keyboards(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS japanese_texts (
                id SERIAL PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium',
                type VARCHAR(20) DEFAULT 'mixed'
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wubi_roots (
                id SERIAL PRIMARY KEY,
                character_val VARCHAR(32) NOT NULL,
                code VARCHAR(32) NOT NULL,
                position VARCHAR(64) NOT NULL,
                description TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS user_progress (
                id SERIAL PRIMARY KEY,
                user_name VARCHAR(64) NOT NULL,
                lesson_id INT NOT NULL,
                accuracy FLOAT NOT NULL,
                score INT NOT NULL,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS key_radicals (
                id SERIAL PRIMARY KEY,
                key_char VARCHAR(4) NOT NULL UNIQUE,
                radicals TEXT NOT NULL,
                formula TEXT,
                description TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 插入示例数据
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM lessons")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if count == 0 {
            let lessons = [
                ("人", "WG", "练习人字的五笔编码。"),
                ("日", "KH", "练习日字的五笔编码。"),
                ("山", "FQ", "练习山字的五笔编码。"),
            ];

            for (char, code, desc) in lessons {
                sqlx::query(
                    "INSERT INTO lessons (character_val, code, description) VALUES ($1, $2, $3)",
                )
                .bind(char)
                .bind(code)
                .bind(desc)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        let art_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM articles")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if art_count == 0 {
            let articles = [
                (
                    "练习文章一",
                    "五笔字型是一种高效的中文输入法，通过拆分汉字为基本字根进行输入。",
                    "easy",
                ),
                (
                    "练习文章二",
                    "学习五笔需要掌握字根分布和拆字规则，多加练习才能熟练运用。",
                    "medium",
                ),
                (
                    "练习文章三",
                    "汉字的结构复杂多样，五笔输入法按照汉字的笔画和结构规律进行编码。",
                    "hard",
                ),
            ];

            for (title, content, difficulty) in articles {
                sqlx::query(
                    "INSERT INTO articles (title, content, difficulty) VALUES ($1, $2, $3)",
                )
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        let root_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wubi_roots")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if root_count == 0 {
            let roots = [
                ("一", "GGLL", "G区第一键", "横区起首字根"),
                ("丿", "TTLL", "T区第一键", "撇区起首字根"),
                ("丨", "HHLL", "H区第一键", "竖区起首字根"),
                ("丶", "YYLL", "Y区第一键", "捺区起首字根"),
                ("乙", "NNLL", "N区第一键", "折区起首字根"),
                ("九", "VTNG", "V区第二键", "字根：乙"),
                ("力", "LTNN", "L键", "字根：力"),
                ("乃", "DETN", "N键", "字根：乃"),
                ("刀", "VNTE", "V键", "字根：刀"),
                ("卜", "HHYD", "H键", "字根：卜"),
            ];

            for (char, code, position, desc) in roots {
                sqlx::query(
                    "INSERT INTO wubi_roots (character_val, code, position, description) VALUES ($1, $2, $3, $4)"
                )
                .bind(char)
                .bind(code)
                .bind(position)
                .bind(desc)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        let key_radical_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM key_radicals")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if key_radical_count == 0 {
            let key_radicals = [
                (
                    "g",
                    "王、一、五、戋",
                    "王旁青头戋（兼）五一",
                    "G区横区第一键，包含横笔和戈字根",
                ),
                (
                    "f",
                    "土、士、二、干、十、寸、雨",
                    "土士二干十寸雨",
                    "F区横区第二键，包含土字根",
                ),
                (
                    "d",
                    "大、犬、三、古、石、厂",
                    "大犬三（古）石厂",
                    "D区横区第三键，包含大字根",
                ),
                ("s", "木、丁、西", "木丁西", "S区横区第四键，包含木字根"),
                (
                    "a",
                    "工、戈、艹、七、廿",
                    "工戈草头右框七",
                    "A区横区第五键，包含工字根",
                ),
                (
                    "h",
                    "目、止、卜、虍、上",
                    "目具上止卜虎皮",
                    "H区竖区第一键，包含目字根",
                ),
                (
                    "j",
                    "日、早、虫、刂、竖",
                    "日早两竖与虫依",
                    "J区竖区第二键，包含日字根",
                ),
                ("k", "口、川", "口与川，字根稀", "K区竖区第三键，包含口字根"),
                (
                    "l",
                    "田、甲、四、车、囗",
                    "田甲方框四车里",
                    "L区竖区第四键，包含田字根",
                ),
                (
                    "m",
                    "山、由、贝、几",
                    "山由贝，下框几",
                    "M区竖区第五键，包含山字根",
                ),
                (
                    "t",
                    "禾、竹、丿、彳、攵",
                    "禾竹一撇双人立",
                    "T区撇区第一键，包含禾字根",
                ),
                (
                    "r",
                    "白、手、斤、牛",
                    "白手看头三二斤",
                    "R区撇区第二键，包含白字根",
                ),
                (
                    "e",
                    "舟、用、月、豕、衣",
                    "舟用乃月豕（家）衣",
                    "E区撇区第三键，包含月字根",
                ),
                (
                    "w",
                    "人、八、亻",
                    "人八登头单人几",
                    "W区撇区第四键，包含人字根",
                ),
                (
                    "q",
                    "金、饣、勹、儿、夕",
                    "金勺缺点无尾鱼，犬旁留叉",
                    "Q区撇区第五键，包含金字根",
                ),
                (
                    "y",
                    "言、文、方、广、丶",
                    "言文方广在四一，高头一捺谁人去",
                    "Y区捺区第一键，包含言字根",
                ),
                (
                    "u",
                    "立、辛、六、门、疒",
                    "立辛两点六门疒（病）",
                    "U区捺区第二键，包含立字根",
                ),
                (
                    "i",
                    "氵（三点水）、小",
                    "水旁兴头小倒立",
                    "I区捺区第三键，包含水字根",
                ),
                (
                    "o",
                    "火、米、灬",
                    "火业头，四点米",
                    "O区捺区第四键，包含火字根",
                ),
                (
                    "p",
                    "之、宀（宝盖）、冖、礻、衤",
                    "之字军盖建道底，摘礻衤",
                    "P区捺区第五键，包含之字根",
                ),
                (
                    "n",
                    "已、己、巳、尸、心、羽",
                    "已半巳满不出己，左框折尸心和羽",
                    "N区折区第一键，包含已字根",
                ),
                (
                    "b",
                    "子、耳、了、也、卩",
                    "子耳了也框向上",
                    "B区折区第二键，包含子字根",
                ),
                (
                    "v",
                    "女、刀、九、臼",
                    "女刀九臼山朝西",
                    "V区折区第三键，包含女字根",
                ),
                (
                    "c",
                    "又、巴、马、厶",
                    "又巴马，丢矢矣",
                    "C区折区第四键，包含又字根",
                ),
                (
                    "x",
                    "幺、母、弓、匕",
                    "慈母无心弓和匕，幼无力",
                    "X区折区第五键，包含丝字根",
                ),
            ];

            for (key, radicals, formula, desc) in key_radicals {
                sqlx::query(
                    "INSERT INTO key_radicals (key_char, radicals, formula, description) VALUES ($1, $2, $3, $4)"
                )
                .bind(key)
                .bind(radicals)
                .bind(formula)
                .bind(desc)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        let english_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM english_texts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if english_count == 0 {
            let english_texts = [
                (
                    "基础练习",
                    "the quick brown fox jumps over the lazy dog",
                    "easy",
                ),
                (
                    "常用句子",
                    "hello world this is a typing practice text for english learning",
                    "easy",
                ),
                (
                    "进阶练习",
                    "practice makes perfect keep typing to improve your speed and accuracy",
                    "medium",
                ),
                (
                    "高级练习",
                    "the five boxing wizards jump quickly at dawn every single day",
                    "hard",
                ),
            ];

            for (title, content, difficulty) in english_texts {
                sqlx::query(
                    "INSERT INTO english_texts (title, content, difficulty) VALUES ($1, $2, $3)",
                )
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化日语键盘布局数据
        let japanese_keyboard_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM japanese_keyboards")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if japanese_keyboard_count == 0 {
            let keyboard_layouts = [(
                "QWERTY Japanese",
                "标准QWERTY键盘的日语布局，使用罗马字输入",
            )];

            for (layout_name, description) in keyboard_layouts {
                sqlx::query(
                    "INSERT INTO japanese_keyboards (layout_name, description) VALUES ($1, $2)",
                )
                .bind(layout_name)
                .bind(description)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化日语字符数据
        let japanese_character_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM japanese_characters")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if japanese_character_count == 0 {
            let japanese_chars = [
                ("あ", "a", "a", 1),
                ("い", "i", "i", 1),
                ("う", "u", "u", 1),
                ("え", "e", "e", 1),
                ("お", "o", "o", 1),
                ("か", "ka", "k", 1),
                ("き", "ki", "k", 1),
                ("く", "ku", "k", 1),
                ("け", "ke", "k", 1),
                ("こ", "ko", "k", 1),
                ("さ", "sa", "s", 1),
                ("し", "shi", "s", 1),
                ("す", "su", "s", 1),
                ("せ", "se", "s", 1),
                ("そ", "so", "s", 1),
            ];

            for (character, romanji, keyboard_key, layout_id) in japanese_chars {
                sqlx::query(
                    "INSERT INTO japanese_characters (character, romanji, keyboard_key, layout_id) VALUES ($1, $2, $3, $4)"
                )
                .bind(character)
                .bind(romanji)
                .bind(keyboard_key)
                .bind(layout_id)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化日语文章数据
        let japanese_text_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM japanese_texts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if japanese_text_count == 0 {
            let japanese_texts = [
                ("平假名基础", "あいうえおかきくけこさしすせそ", "easy", "hiragana"),
                ("片假名基础", "アイウエオカキクケコサシスセソ", "easy", "katakana"),
                ("汉字基础", "一二三四五六七八九十", "easy", "kanji"),
                ("混合练习", "こんにちは、元気ですか？私は元気です。", "easy", "mixed"),
                ("平假名进阶", "いろはにほへとちりぬるをわがよたれぞつねならむ", "medium", "hiragana"),
                ("片假名进阶", "イロハニホヘトチリヌルヲワガヨタレゾツネナラム", "medium", "katakana"),
                ("汉字进阶", "日本語の勉強は楽しいです。毎日勉強しています。", "medium", "kanji"),
                ("混合进阶", "東京は日本の首都です。人口が多くてにぎやかです。", "medium", "mixed"),
                ("高级平假名", "むかしむかしあるところにおじいさんとおばあさんがいました", "hard", "hiragana"),
                ("高级片假名", "ムカシムカシアルトコロニオジイサントオバアサンガイマシタ", "hard", "katakana"),
                ("高级汉字", "日本国は、東アジアの島国であり、首都は東京です。", "hard", "kanji"),
                ("高级混合", "今日はいい天気ですね。公園に行きませんか？", "hard", "mixed"),
            ];

            for (title, content, difficulty, r#type) in japanese_texts {
                sqlx::query(
                    "INSERT INTO japanese_texts (title, content, difficulty, type) VALUES ($1, $2, $3, $4)",
                )
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .bind(r#type)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        let char_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM wubi_characters")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if char_count == 0 {
            let dict_path = "data/wubi86_converted.json";
            if Path::new(dict_path).exists() {
                let content = fs::read_to_string(dict_path)
                    .map_err(|e| format!("Failed to read wubi dictionary: {}", e))?;
                let entries: Vec<WubiDictEntry> = serde_json::from_str(&content)
                    .map_err(|e| format!("Failed to parse wubi dictionary: {}", e))?;

                let entries_len = entries.len();
                println!(
                    "Importing {} wubi dictionary entries to PostgreSQL...",
                    entries_len
                );

                for entry in entries {
                    sqlx::query(
                        "INSERT INTO wubi_characters (character_val, simple_code, full_code, pinyin, remark) VALUES ($1, $2, $3, $4, $5) ON CONFLICT (character_val) DO NOTHING"
                    )
                    .bind(&entry.character)
                    .bind(&entry.simple_code)
                    .bind(&entry.full_code)
                    .bind(&entry.pinyin)
                    .bind(&entry.remark)
                    .execute(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?;
                }

                println!("Wubi dictionary import complete: {} entries", entries_len);
            }
        }

        // 创建日语五十音表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS japanese_gojuon (
                id SERIAL PRIMARY KEY,
                "character" VARCHAR(10) NOT NULL UNIQUE,
                romanji VARCHAR(50) NOT NULL,
                "row" VARCHAR(20) NOT NULL,
                "column" VARCHAR(20) NOT NULL,
                pronunciation VARCHAR(100) NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 创建日语汉字表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS japanese_kanji (
                id SERIAL PRIMARY KEY,
                kanji VARCHAR(10) NOT NULL UNIQUE,
                reading VARCHAR(100) NOT NULL,
                meaning VARCHAR(255) NOT NULL,
                level VARCHAR(20) NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        let english_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM english_texts")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if english_count == 0 {
            let english_texts = [
                (
                    "基础练习",
                    "the quick brown fox jumps over the lazy dog",
                    "easy",
                ),
                (
                    "常用句子",
                    "hello world this is a typing practice text for english learning",
                    "easy",
                ),
                (
                    "进阶练习",
                    "practice makes perfect keep typing to improve your speed and accuracy",
                    "medium",
                ),
                (
                    "高级练习",
                    "the five boxing wizards jump quickly at dawn every single day",
                    "hard",
                ),
            ];

            for (title, content, difficulty) in english_texts {
                sqlx::query(
                    "INSERT INTO english_texts (title, content, difficulty) VALUES ($1, $2, $3)",
                )
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化日语键盘布局数据
        let japanese_keyboard_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM japanese_keyboards")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if japanese_keyboard_count == 0 {
            let keyboard_layouts = [(
                "QWERTY Japanese",
                "标准QWERTY键盘的日语布局，使用罗马字输入",
            )];

            for (layout_name, description) in keyboard_layouts {
                sqlx::query(
                    "INSERT INTO japanese_keyboards (layout_name, description) VALUES ($1, $2)",
                )
                .bind(layout_name)
                .bind(description)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化日语字符数据
        let japanese_character_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM japanese_characters")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if japanese_character_count == 0 {
            let japanese_chars = [
                ("あ", "a", "a", 1),
                ("い", "i", "i", 1),
                ("う", "u", "u", 1),
                ("え", "e", "e", 1),
                ("お", "o", "o", 1),
                ("か", "ka", "k", 1),
                ("き", "ki", "k", 1),
                ("く", "ku", "k", 1),
                ("け", "ke", "k", 1),
                ("こ", "ko", "k", 1),
                ("さ", "sa", "s", 1),
                ("し", "shi", "s", 1),
                ("す", "su", "s", 1),
                ("せ", "se", "s", 1),
                ("そ", "so", "s", 1),
            ];

            for (character, romanji, keyboard_key, layout_id) in japanese_chars {
                sqlx::query(
                    "INSERT INTO japanese_characters (character, romanji, keyboard_key, layout_id) VALUES ($1, $2, $3, $4)"
                )
                .bind(character)
                .bind(romanji)
                .bind(keyboard_key)
                .bind(layout_id)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化日语五十音数据
        let japanese_gojuon_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM japanese_gojuon")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if japanese_gojuon_count == 0 {
            let gojuon_data = [
                ("あ", "a", "あ行", "あ段", "ア"),
                ("い", "i", "あ行", "い段", "イ"),
                ("う", "u", "あ行", "う段", "ウ"),
                ("え", "e", "あ行", "え段", "エ"),
                ("お", "o", "あ行", "お段", "オ"),
                ("か", "ka", "か行", "あ段", "カ"),
                ("き", "ki", "か行", "い段", "キ"),
                ("く", "ku", "か行", "う段", "ク"),
                ("け", "ke", "か行", "え段", "ケ"),
                ("こ", "ko", "か行", "お段", "コ"),
                ("さ", "sa", "さ行", "あ段", "サ"),
                ("し", "shi", "さ行", "い段", "シ"),
                ("す", "su", "さ行", "う段", "ス"),
                ("せ", "se", "さ行", "え段", "セ"),
                ("そ", "so", "さ行", "お段", "ソ"),
                ("た", "ta", "た行", "あ段", "タ"),
                ("ち", "chi", "た行", "い段", "チ"),
                ("つ", "tsu", "た行", "う段", "ツ"),
                ("て", "te", "た行", "え段", "テ"),
                ("と", "to", "た行", "お段", "ト"),
                ("な", "na", "な行", "あ段", "ナ"),
                ("に", "ni", "な行", "い段", "ニ"),
                ("ぬ", "nu", "な行", "う段", "ヌ"),
                ("ね", "ne", "な行", "え段", "ネ"),
                ("の", "no", "な行", "お段", "ノ"),
                ("は", "ha", "は行", "あ段", "ハ"),
                ("ひ", "hi", "は行", "い段", "ヒ"),
                ("ふ", "fu", "は行", "う段", "フ"),
                ("へ", "he", "は行", "え段", "ヘ"),
                ("ほ", "ho", "は行", "お段", "ホ"),
                ("ま", "ma", "ま行", "あ段", "マ"),
                ("み", "mi", "ま行", "い段", "ミ"),
                ("む", "mu", "ま行", "う段", "ム"),
                ("め", "me", "ま行", "え段", "メ"),
                ("も", "mo", "ま行", "お段", "モ"),
                ("や", "ya", "や行", "あ段", "ヤ"),
                ("ゆ", "yu", "や行", "う段", "ユ"),
                ("よ", "yo", "や行", "お段", "ヨ"),
                ("ら", "ra", "ら行", "あ段", "ラ"),
                ("り", "ri", "ら行", "い段", "リ"),
                ("る", "ru", "ら行", "う段", "ル"),
                ("れ", "re", "ら行", "え段", "レ"),
                ("ろ", "ro", "ら行", "お段", "ロ"),
                ("わ", "wa", "わ行", "あ段", "ワ"),
                ("を", "wo", "わ行", "お段", "ヲ"),
                ("ん", "n", "ん行", "ん段", "ン"),
            ];

            for (character, romanji, row, column, pronunciation) in gojuon_data {
                sqlx::query(
                    "INSERT INTO japanese_gojuon (\"character\", romanji, \"row\", \"column\", pronunciation) VALUES ($1, $2, $3, $4, $5)"
                )
                .bind(character)
                .bind(romanji)
                .bind(row)
                .bind(column)
                .bind(pronunciation)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化日语汉字数据
        let japanese_kanji_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM japanese_kanji")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if japanese_kanji_count == 0 {
            let kanji_data = [
                ("一", "いち", "一", "N5"),
                ("二", "に", "二", "N5"),
                ("三", "さん", "三", "N5"),
                ("四", "よん", "四", "N5"),
                ("五", "ご", "五", "N5"),
                ("六", "ろく", "六", "N5"),
                ("七", "なな", "七", "N5"),
                ("八", "はち", "八", "N5"),
                ("九", "きゅう", "九", "N5"),
                ("十", "じゅう", "十", "N5"),
                ("日", "にち", "日", "N5"),
                ("月", "つき", "月", "N5"),
                ("火", "ひ", "火", "N5"),
                ("水", "みず", "水", "N5"),
                ("木", "き", "木", "N5"),
                ("金", "かね", "金", "N5"),
                ("土", "つち", "土", "N5"),
                ("人", "ひと", "人", "N5"),
                ("大", "おお", "大", "N5"),
                ("小", "ちい", "小", "N5"),
            ];

            for (kanji, reading, meaning, level) in kanji_data {
                sqlx::query(
                    "INSERT INTO japanese_kanji (kanji, reading, meaning, level) VALUES ($1, $2, $3, $4)"
                )
                .bind(kanji)
                .bind(reading)
                .bind(meaning)
                .bind(level)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 创建注音键盘布局表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bopomofo_keyboards (
                id SERIAL PRIMARY KEY,
                layout_name VARCHAR(100) NOT NULL UNIQUE,
                description TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 创建注音字符表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bopomofo_characters (
                id SERIAL PRIMARY KEY,
                "character" VARCHAR(10) NOT NULL,
                bopomofo VARCHAR(50) NOT NULL,
                keyboard_key VARCHAR(10) NOT NULL,
                layout_id INT NOT NULL,
                FOREIGN KEY (layout_id) REFERENCES bopomofo_keyboards(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 创建繁体中文文章表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS traditional_chinese_texts (
                id SERIAL PRIMARY KEY,
                title VARCHAR(255) NOT NULL,
                content TEXT NOT NULL,
                difficulty VARCHAR(10) DEFAULT 'medium'
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        // 初始化注音键盘布局数据
        let bopomofo_keyboard_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM bopomofo_keyboards")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if bopomofo_keyboard_count == 0 {
            sqlx::query(
                "INSERT INTO bopomofo_keyboards (layout_name, description) VALUES ($1, $2)",
            )
            .bind("Bopomofo Keyboard")
            .bind("标准注音键盘布局，使用注音符号输入")
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        // 初始化注音字符数据
        let bopomofo_character_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM bopomofo_characters")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if bopomofo_character_count == 0 {
            let bopomofo_chars = [
                ("ㄅ", "b", "b", 1), ("ㄆ", "p", "p", 1), ("ㄇ", "m", "m", 1),
                ("ㄈ", "f", "f", 1), ("ㄉ", "d", "d", 1), ("ㄊ", "t", "t", 1),
                ("ㄋ", "n", "n", 1), ("ㄌ", "l", "l", 1), ("ㄍ", "g", "g", 1),
                ("ㄎ", "k", "k", 1), ("ㄏ", "h", "h", 1), ("ㄐ", "j", "j", 1),
                ("ㄑ", "q", "q", 1), ("ㄒ", "x", "x", 1), ("ㄓ", "zh", "z", 1),
                ("ㄔ", "ch", "c", 1), ("ㄕ", "sh", "s", 1), ("ㄖ", "r", "r", 1),
                ("ㄗ", "z", "z", 1), ("ㄘ", "c", "c", 1), ("ㄙ", "s", "s", 1),
                ("ㄧ", "i", "i", 1), ("ㄨ", "u", "u", 1), ("ㄩ", "ü", "v", 1),
                ("ㄚ", "a", "a", 1), ("ㄛ", "o", "o", 1), ("ㄜ", "e", "e", 1),
                ("ㄝ", "ê", "e", 1), ("ㄞ", "ai", "a", 1), ("ㄟ", "ei", "e", 1),
                ("ㄠ", "ao", "a", 1), ("ㄡ", "ou", "o", 1), ("ㄢ", "an", "a", 1),
                ("ㄣ", "en", "e", 1), ("ㄤ", "ang", "a", 1), ("ㄥ", "eng", "e", 1),
                ("ㄦ", "er", "e", 1),
            ];

            for (character, bopomofo, keyboard_key, layout_id) in bopomofo_chars {
                sqlx::query(
                    "INSERT INTO bopomofo_characters (\"character\", bopomofo, keyboard_key, layout_id) VALUES ($1, $2, $3, $4)"
                )
                .bind(character)
                .bind(bopomofo)
                .bind(keyboard_key)
                .bind(layout_id)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        // 初始化繁体中文文章数据
        let traditional_chinese_text_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM traditional_chinese_texts")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if traditional_chinese_text_count == 0 {
            let traditional_chinese_texts = [
                ("基础练习", "一二人三四五六七八九十", "easy"),
                ("常用句子", "你好，歡迎光臨！請問有什麼可以幫您的嗎？", "easy"),
                ("进阶练习", "中華文化源遠流長，博大精深，值得我們好好學習。", "medium"),
                ("高级练习", "臺灣是一個美麗的島嶼，擁有豐富的自然資源和人文景觀。", "hard"),
            ];

            for (title, content, difficulty) in traditional_chinese_texts {
                sqlx::query(
                    "INSERT INTO traditional_chinese_texts (title, content, difficulty) VALUES ($1, $2, $3)"
                )
                .bind(title)
                .bind(content)
                .bind(difficulty)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }

    async fn get_wubi_code(&self, character: &str) -> Result<WubiCharacter, String> {
        sqlx::query_as::<_, (i32, String, String, String, String, String)>(
            "SELECT id, character_val, simple_code, full_code, pinyin, remark FROM wubi_characters WHERE character_val = $1"
        )
        .bind(character)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Character not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, character, simple_code, full_code, pinyin, remark)| WubiCharacter { id, character, simple_code, full_code, pinyin, remark })
    }

    async fn get_all_wubi_characters(&self) -> Result<Vec<WubiCharacter>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String, String)>(
            "SELECT id, character_val, simple_code, full_code, pinyin, remark FROM wubi_characters ORDER BY character_val"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, simple_code, full_code, pinyin, remark)| WubiCharacter {
            id, character, simple_code, full_code, pinyin, remark
        }).collect())
    }

    async fn get_lessons(&self) -> Result<Vec<Lesson>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, character_val, code, description FROM lessons ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, character, code, description)| Lesson {
                    id,
                    character,
                    code,
                    description,
                })
                .collect()
        })
    }

    async fn get_lesson_by_id(&self, id: i32) -> Result<Lesson, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, character_val, code, description FROM lessons WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Lesson not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, character, code, description)| Lesson {
            id,
            character,
            code,
            description,
        })
    }

    async fn create_lesson(
        &self,
        character: &str,
        code: &str,
        description: &str,
    ) -> Result<Lesson, String> {
        let lesson = sqlx::query_as::<_, (i32, String, String, String)>(
            "INSERT INTO lessons (character_val, code, description) VALUES ($1, $2, $3) RETURNING id, character_val, code, description"
        )
        .bind(character)
        .bind(code)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|(id, character, code, description)| Lesson {
            id, character, code, description
        });
        lesson
    }

    async fn get_articles(&self) -> Result<Vec<Article>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM articles ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, title, content, difficulty)| Article {
                    id,
                    title,
                    content,
                    difficulty,
                })
                .collect()
        })
    }

    async fn get_article_by_id(&self, id: i32) -> Result<Article, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM articles WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Article not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, title, content, difficulty)| Article {
            id,
            title,
            content,
            difficulty,
        })
    }

    async fn create_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let article = sqlx::query_as::<_, (i32, String, String, String)>(
            "INSERT INTO articles (title, content, difficulty) VALUES ($1, $2, $3) RETURNING id, title, content, difficulty"
        )
        .bind(title)
        .bind(content)
        .bind(difficulty)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|(id, title, content, difficulty)| Article {
            id, title, content, difficulty
        });
        article
    }

    async fn update_article(
        &self,
        id: i32,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let article = sqlx::query_as::<_, (i32, String, String, String)>(
            "UPDATE articles SET title = $1, content = $2, difficulty = $3 WHERE id = $4 RETURNING id, title, content, difficulty"
        )
        .bind(title)
        .bind(content)
        .bind(difficulty)
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Article not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, title, content, difficulty)| Article {
            id, title, content, difficulty
        });
        article
    }

    async fn delete_article(&self, id: i32) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM articles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if result.rows_affected() == 0 {
            Err("Article not found".to_string())
        } else {
            Ok(())
        }
    }

    async fn get_custom_articles(&self) -> Result<Vec<Article>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM custom_articles ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, title, content, difficulty)| Article {
                    id,
                    title,
                    content,
                    difficulty,
                })
                .collect()
        })
    }

    async fn create_custom_article(
        &self,
        title: &str,
        content: &str,
        difficulty: &str,
    ) -> Result<Article, String> {
        let article = sqlx::query_as::<_, (i32, String, String, String)>(
            "INSERT INTO custom_articles (title, content, difficulty) VALUES ($1, $2, $3) RETURNING id, title, content, difficulty"
        )
        .bind(title)
        .bind(content)
        .bind(difficulty)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|(id, title, content, difficulty)| Article {
            id, title, content, difficulty
        });
        article
    }

    async fn delete_custom_article(&self, id: i32) -> Result<(), String> {
        let result = sqlx::query("DELETE FROM custom_articles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        if result.rows_affected() == 0 {
            Err("Custom article not found".to_string())
        } else {
            Ok(())
        }
    }

    async fn update_wubi_code(
        &self,
        character: &str,
        new_code: &str,
    ) -> Result<WubiCharacter, String> {
        let result =
            sqlx::query("UPDATE wubi_characters SET full_code = $1 WHERE character_val = $2")
                .bind(new_code)
                .bind(character)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;

        if result.rows_affected() == 0 {
            let _ = sqlx::query(
                "INSERT INTO wubi_characters (character_val, full_code) VALUES ($1, $2)",
            )
            .bind(character)
            .bind(new_code)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        }

        Ok(WubiCharacter {
            id: 0,
            character: character.to_string(),
            simple_code: String::new(),
            full_code: new_code.to_string(),
            pinyin: String::new(),
            remark: String::new(),
        })
    }

    async fn get_wubi_roots(&self) -> Result<Vec<WubiRoot>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots ORDER BY position"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, code, position, description)| WubiRoot {
            id, character, code, position, description
        }).collect())
    }

    async fn get_wubi_root_by_id(&self, id: i32) -> Result<WubiRoot, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => "Wubi root not found".to_string(),
            _ => e.to_string(),
        })
        .map(|(id, character, code, position, description)| WubiRoot {
            id,
            character,
            code,
            position,
            description,
        })
    }

    async fn create_wubi_root(
        &self,
        character: &str,
        code: &str,
        position: &str,
        description: &str,
    ) -> Result<WubiRoot, String> {
        let root = sqlx::query_as::<_, (i32, String, String, String, String)>(
            "INSERT INTO wubi_roots (character_val, code, position, description) VALUES ($1, $2, $3, $4) RETURNING id, character_val, code, position, description"
        )
        .bind(character)
        .bind(code)
        .bind(position)
        .bind(description)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|(id, character, code, position, description)| WubiRoot {
            id, character, code, position, description
        });
        root
    }

    async fn search_wubi_root(&self, character: &str) -> Result<Option<WubiRoot>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, character_val, code, position, description FROM wubi_roots WHERE character_val = $1"
        )
        .bind(character)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(
            result.map(|(id, character, code, position, description)| WubiRoot {
                id,
                character,
                code,
                position,
                description,
            }),
        )
    }

    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, String> {
        let result = sqlx::query_as::<
            _,
            (i32, String, String, String, Option<chrono::NaiveDateTime>),
        >(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(
            result.map(|(id, username, email, password_hash, created_at)| User {
                id,
                username,
                email,
                password_hash,
                created_at: created_at
                    .map(|dt| dt.and_utc().to_rfc3339())
                    .unwrap_or_default(),
            }),
        )
    }

    async fn get_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
        let result =
            sqlx::query_as::<_, (i32, String, String, String, Option<chrono::NaiveDateTime>)>(
                "SELECT id, username, email, password_hash, created_at FROM users WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(
            result.map(|(id, username, email, password_hash, created_at)| User {
                id,
                username,
                email,
                password_hash,
                created_at: created_at
                    .map(|dt| dt.and_utc().to_rfc3339())
                    .unwrap_or_default(),
            }),
        )
    }

    async fn create_user(
        &self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, String> {
        let user = sqlx::query_as::<_, (i32, String, String, String, Option<chrono::NaiveDateTime>)>(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email, password_hash, created_at"
        )
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|(id, username, email, password_hash, created_at)| User {
            id, username, email, password_hash, created_at: created_at.map(|dt| dt.and_utc().to_rfc3339()).unwrap_or_default()
        });
        user
    }

    async fn save_progress(
        &self,
        user_name: &str,
        lesson_id: i32,
        accuracy: f32,
        score: i32,
    ) -> Result<(), String> {
        sqlx::query(
            "INSERT INTO user_progress (user_name, lesson_id, accuracy, score) VALUES ($1, $2, $3, $4)"
        )
        .bind(user_name)
        .bind(lesson_id)
        .bind(accuracy)
        .bind(score)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn get_key_radicals(&self) -> Result<Vec<KeyRadical>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, key_char, radicals, formula, description FROM key_radicals ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(
                    |(id, key_char, radicals, formula, description)| KeyRadical {
                        id,
                        key_char,
                        radicals,
                        formula,
                        description,
                    },
                )
                .collect()
        })
    }

    async fn get_key_radical_by_key(&self, key_char: &str) -> Result<Option<KeyRadical>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, key_char, radicals, formula, description FROM key_radicals WHERE key_char = $1"
        )
        .bind(key_char)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.map(
            |(id, key_char, radicals, formula, description)| KeyRadical {
                id,
                key_char,
                radicals,
                formula,
                description,
            },
        ))
    }

    async fn get_english_texts(&self) -> Result<Vec<EnglishText>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM english_texts ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, title, content, difficulty)| EnglishText {
                    id,
                    title,
                    content,
                    difficulty,
                })
                .collect()
        })
    }

    async fn get_japanese_texts(&self) -> Result<Vec<JapaneseText>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, title, content, difficulty, type FROM japanese_texts ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, title, content, difficulty, r#type)| JapaneseText {
                    id,
                    title,
                    content,
                    difficulty,
                    r#type,
                })
                .collect()
        })
    }

    async fn get_japanese_keyboards(&self) -> Result<Vec<JapaneseKeyboard>, String> {
        sqlx::query_as::<_, (i32, String, String)>(
            "SELECT id, layout_name, description FROM japanese_keyboards ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, layout_name, description)| JapaneseKeyboard {
                    id,
                    layout_name,
                    description,
                })
                .collect()
        })
    }

    async fn get_japanese_characters(&self) -> Result<Vec<JapaneseCharacter>, String> {
        sqlx::query_as::<_, (i32, String, String, String, i32)>(
            "SELECT id, `character`, romanji, keyboard_key, layout_id FROM japanese_characters ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, romanji, keyboard_key, layout_id)| JapaneseCharacter {
            id, character, romanji, keyboard_key, layout_id
        }).collect())
    }

    async fn get_traditional_chinese_texts(&self) -> Result<Vec<TraditionalChineseText>, String> {
        sqlx::query_as::<_, (i32, String, String, String)>(
            "SELECT id, title, content, difficulty FROM traditional_chinese_texts ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, title, content, difficulty)| TraditionalChineseText {
                    id,
                    title,
                    content,
                    difficulty,
                })
                .collect()
        })
    }

    async fn get_bopomofo_keyboards(&self) -> Result<Vec<BopomofoKeyboard>, String> {
        sqlx::query_as::<_, (i32, String, String)>(
            "SELECT id, layout_name, description FROM bopomofo_keyboards ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, layout_name, description)| BopomofoKeyboard {
                    id,
                    layout_name,
                    description,
                })
                .collect()
        })
    }

    async fn get_bopomofo_characters(&self) -> Result<Vec<BopomofoCharacter>, String> {
        sqlx::query_as::<_, (i32, String, String, String, i32)>(
            "SELECT id, \"character\", bopomofo, keyboard_key, layout_id FROM bopomofo_characters ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, bopomofo, keyboard_key, layout_id)| BopomofoCharacter {
            id, character, bopomofo, keyboard_key, layout_id
        }).collect())
    }

    async fn get_japanese_gojuon(&self) -> Result<Vec<JapaneseGojuon>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String, String)>(
            "SELECT id, \"character\", romanji, \"row\", \"column\", pronunciation FROM japanese_gojuon ORDER BY id"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| rows.into_iter().map(|(id, character, romanji, row, column, pronunciation)| JapaneseGojuon {
            id, character, romanji, row, column, pronunciation
        }).collect())
    }

    async fn get_japanese_kanji(&self) -> Result<Vec<JapaneseKanji>, String> {
        sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, kanji, reading, meaning, level FROM japanese_kanji ORDER BY id",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())
        .map(|rows| {
            rows.into_iter()
                .map(|(id, kanji, reading, meaning, level)| JapaneseKanji {
                    id,
                    kanji,
                    reading,
                    meaning,
                    level,
                })
                .collect()
        })
    }

    async fn get_japanese_kanji_by_char(
        &self,
        kanji: &str,
    ) -> Result<Option<JapaneseKanji>, String> {
        let result = sqlx::query_as::<_, (i32, String, String, String, String)>(
            "SELECT id, kanji, reading, meaning, level FROM japanese_kanji WHERE kanji = ?",
        )
        .bind(kanji)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(
            result.map(|(id, kanji, reading, meaning, level)| JapaneseKanji {
                id,
                kanji,
                reading,
                meaning,
                level,
            }),
        )
    }
}
