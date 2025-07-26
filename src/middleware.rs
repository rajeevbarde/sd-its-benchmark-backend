pub mod logging;
pub mod cors;
pub mod timeout;
pub mod size_limit;
pub mod security_headers;

use axum::Router;

pub fn apply_middleware(router: Router) -> Router {
    use tower_http::trace::TraceLayer;
    use tower_http::cors::CorsLayer;
    use tower_http::limit::RequestBodyLimitLayer;
    use tower_http::set_header::SetResponseHeaderLayer;
    use axum::http::header::{HeaderName, HeaderValue};

    router
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10 MB
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            |_: &_| Some(HeaderValue::from_static("nosniff")),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            |_: &_| Some(HeaderValue::from_static("DENY")),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-xss-protection"),
            |_: &_| Some(HeaderValue::from_static("1; mode=block")),
        ))
}
