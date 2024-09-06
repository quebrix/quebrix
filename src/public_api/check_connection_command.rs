

use actix_web::{
    http::header::HeaderMap, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer,
};
use super::server::ApiResponse;

pub async fn check_connection() -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::ok("Pong"))
}