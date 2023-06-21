use actix_web::{HttpResponse, Responder};

/// The endpoint is used to check if the server is running and responding to
/// requests. When a client sends a GET request to this endpoint, the server
/// responds with an HTTP 200 OK status code.
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}
