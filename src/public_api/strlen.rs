use super::server::ApiResponse;
use crate::cache::strlen::Strlen;
use crate::{cache::Cache, creds::cred_manager::CredsManager};
use actix_web::{http::header::HeaderMap, web, HttpRequest, HttpResponse};
use base64::decode;
use std::sync::{Arc, Mutex};
pub async fn strlen(
    cache: web::Data<Arc<Mutex<Cache>>>,
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    info: web::Path<(String, String)>,
    req: HttpRequest,
) -> HttpResponse {
    let (cluster, key) = info.into_inner();
    let headers: &HeaderMap = req.headers();
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    let decoded_bytes = decode(auth).expect("Failed to decode Base64 string");
    let decoded_credentials =
        std::str::from_utf8(&decoded_bytes).expect("Failed to convert bytes to string");
    let creds_vec: Vec<&str> = decoded_credentials.split(":").collect();
    let username = creds_vec.get(0).unwrap();
    let password = creds_vec.get(1).unwrap();
    if !creds.lock().unwrap().authenticate(username, password) {
        return HttpResponse::Unauthorized().json(ApiResponse::fail("Authentication failed"));
    }
    let result = cache.lock().unwrap().strlen(&cluster, &key);
    HttpResponse::Ok().json(result)
}
