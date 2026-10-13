//! Swagger UI 页面：静态资源内置于 swagger-ui-dist/（编译期嵌入二进制，来源
//! utoipa-swagger-ui-vendored 的 swagger-ui v5.32.6 发行包），
//! OpenAPI 规范由 utoipa 从 handler 注解生成。
//!
//! 页面: /swagger-ui/  规范JSON: /api-docs/openapi.json

use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Router,
};
use rust_embed::Embed;
use utoipa::OpenApi;

#[derive(Embed)]
#[folder = "swagger-ui-dist"]
struct SwaggerAssets;

/// OpenAPI 规范 JSON
pub async fn openapi_json() -> impl IntoResponse {
    axum::Json(crate::openapi::ApiDoc::openapi())
}

/// Swagger UI 首页：引用同目录下的内嵌静态资源，配置指向 /api-docs/openapi.json
pub async fn index() -> Html<&'static str> {
    Html(
        r##"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>五笔打字练习 API 文档</title>
  <link rel="stylesheet" href="/swagger-ui/swagger-ui.css" />
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="/swagger-ui/swagger-ui-bundle.js" charset="UTF-8"></script>
  <script src="/swagger-ui/swagger-ui-standalone-preset.js" charset="UTF-8"></script>
  <script>
    window.onload = () => {
      window.ui = SwaggerUIBundle({
        url: "/api-docs/openapi.json",
        dom_id: "#swagger-ui",
        presets: [SwaggerUIBundle.presets.apis, SwaggerUIStandalonePreset],
        layout: "BaseLayout",
      });
    };
  </script>
</body>
</html>"##,
    )
}

/// 提供 Swagger UI 静态资源（css/js/favicon 等）
pub async fn assets(Path(file): Path<String>) -> Response {
    let file = file.trim_start_matches('/');
    match SwaggerAssets::get(file) {
        Some(asset) => {
            let mime = mime_guess::from_path(file).first_or_octet_stream();
            (
                [(axum::http::header::CONTENT_TYPE, mime.as_ref())],
                asset.data,
            )
                .into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
