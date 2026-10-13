//! 嵌入式前端静态资源服务。
//! 前端构建产物（frontend/dist）在编译期嵌入二进制，由Rust服务直接托管，
//! 无需单独的Web服务器。SPA路由（vue-router history模式）回退到index.html。

use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "frontend/dist"]
struct Assets;

/// 所有未匹配路由的回退处理：
/// - /api开头的路径返回404 JSON
/// - 存在的静态文件直接返回
/// - 其余路径回退到index.html（SPA history路由）
pub async fn serve(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');

    if path.starts_with("api/") {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({"error": "not found"})),
        )
            .into_response();
    }

    if path.is_empty() {
        return index_html_response();
    }

    match Assets::get(path) {
        Some(file) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                file.data,
            )
                .into_response()
        }
        None => index_html_response(),
    }
}

fn index_html_response() -> Response {
    match Assets::get("index.html") {
        Some(file) => (
            [(
                axum::http::header::CONTENT_TYPE,
                "text/html; charset=utf-8",
            )],
            file.data,
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            "frontend/dist not found, run `npm run build` in frontend/ first",
        )
            .into_response(),
    }
}
