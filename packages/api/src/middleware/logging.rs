use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

pub async fn log_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    log::info!("→ {method} {path}");
    let start = Instant::now();
    let response = next.run(req).await;
    let status = response.status();
    let ms = start.elapsed().as_millis();
    if status.is_server_error() {
        log::error!("← {status} {ms}ms  {method} {path}");
    } else if status.is_client_error() {
        log::warn!("← {status} {ms}ms  {method} {path}");
    } else {
        log::info!("← {status} {ms}ms  {method} {path}");
    }
    response
}
