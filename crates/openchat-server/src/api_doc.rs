//! OpenAPI 文档元数据（安全方案、info 等），与 `utoipa` 路由合并。

use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::Modify;
use utoipa::OpenApi;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi
            .components
            .get_or_insert_with(utoipa::openapi::Components::new);
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("HS256 JWT（typ: access）"))
                    .build(),
            ),
        );
    }
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "OpenChat API",
        version = "0.1.0",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    ),
    servers((url = "http://127.0.0.1:8080", description = "Local（默认 PORT）")),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;
