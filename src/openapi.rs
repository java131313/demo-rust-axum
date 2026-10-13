//! OpenAPI 文档定义：聚合所有API端点，供Swagger UI展示
//!
//! Swagger UI页面: /swagger-ui
//! OpenAPI JSON:   /api-docs/openapi.json

use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "五笔打字练习 API",
        description = "Rust Axum + SQLite + Redis 缓存的打字练习后端服务",
        version = "2.0.0"
    ),
    paths(
        crate::wubi::health,
        crate::wubi::get_lessons,
        crate::wubi::get_lesson,
        crate::wubi::create_lesson,
        crate::wubi::post_progress,
        crate::wubi::get_articles,
        crate::wubi::get_article,
        crate::wubi::create_article,
        crate::wubi::update_article,
        crate::wubi::delete_article,
        crate::wubi::get_custom_articles,
        crate::wubi::create_custom_article,
        crate::wubi::update_custom_article,
        crate::wubi::delete_custom_article,
        crate::wubi::get_wubi_roots,
        crate::wubi::get_wubi_root,
        crate::wubi::create_wubi_root,
        crate::wubi::search_wubi_root,
        crate::wubi::get_wubi_code,
        crate::wubi::update_wubi_code_handler,
        crate::wubi::login,
        crate::wubi::logout,
        crate::wubi::register,
        crate::wubi::get_key_radicals,
        crate::wubi::get_key_radical_by_key,
        crate::wubi::get_english_texts,
        crate::wubi::get_japanese_texts,
        crate::wubi::get_japanese_keyboards,
        crate::wubi::get_japanese_characters,
        crate::wubi::get_traditional_chinese_texts,
        crate::wubi::get_bopomofo_keyboards,
        crate::wubi::get_bopomofo_characters,
        crate::wubi::get_japanese_gojuon,
        crate::wubi::get_japanese_kanji,
        crate::wubi::get_japanese_kanji_by_char,
    ),
    components(schemas(
        crate::config::User,
        crate::config::Lesson,
        crate::config::Article,
        crate::config::WubiRoot,
        crate::config::WubiCharacter,
        crate::config::KeyRadical,
        crate::config::EnglishText,
        crate::config::JapaneseText,
        crate::config::JapaneseKeyboard,
        crate::config::JapaneseCharacter,
        crate::config::TraditionalChineseText,
        crate::config::BopomofoKeyboard,
        crate::config::BopomofoCharacter,
        crate::config::JapaneseGojuon,
        crate::config::JapaneseKanji,
        crate::wubi::LoginRequest,
        crate::wubi::RegisterRequest,
        crate::wubi::LoginResponse,
        crate::wubi::NewLesson,
        crate::wubi::ProgressUpdate,
        crate::wubi::NewArticle,
        crate::wubi::NewWubiRoot,
        crate::wubi::UpdateWubiCode,
    )),
    tags(
        (name = "system", description = "系统"),
        (name = "auth", description = "认证"),
        (name = "articles", description = "文章"),
        (name = "lessons", description = "课程"),
        (name = "progress", description = "进度"),
        (name = "wubi", description = "五笔"),
        (name = "texts", description = "练习文本"),
    )
)]
pub struct ApiDoc;
