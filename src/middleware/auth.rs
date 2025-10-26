// Simple authentication middleware placeholder
// In production, implement JWT validation or similar

use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// Simple authentication middleware (placeholder)
/// In production, validate JWT tokens here
pub async fn auth_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    // TODO: Implement proper authentication
    // For now, allow all requests

    // Example JWT validation logic (commented out):
    // let auth_header = req.headers()
    //     .get(axum::http::header::AUTHORIZATION)
    //     .and_then(|header| header.to_str().ok());
    //
    // if let Some(auth_header) = auth_header {
    //     if auth_header.starts_with("Bearer ") {
    //         let token = &auth_header[7..];
    //         // Validate token here
    //         return Ok(next.run(req).await);
    //     }
    // }
    //
    // Err(StatusCode::UNAUTHORIZED)

    Ok(next.run(req).await)
}

/// Extract user ID from request headers
/// This is a helper function for when authentication is implemented
pub fn extract_user_id(req: &Request<Body>) -> Option<String> {
    // TODO: Extract from validated JWT token
    // For now, check for a simple X-User-ID header
    req.headers()
        .get("X-User-ID")
        .and_then(|header| header.to_str().ok())
        .map(|s| s.to_string())
}
