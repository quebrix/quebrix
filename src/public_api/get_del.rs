use super::server::ApiResponse;
use crate::{
    cache::{cache::ResultValue, get::Get, get_del::GetDel, Cache},
    creds::cred_manager::CredsManager,
};
use actix_web::{http::header::HeaderMap, web, HttpRequest, HttpResponse};
use base64::decode;
use std::sync::{Arc, Mutex};
pub async fn get_del(
    cache: web::Data<Arc<Mutex<Cache>>>,
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    info: web::Path<(String, String)>,
    req: HttpRequest,
) -> HttpResponse {
    let (cluster, key) = info.into_inner();
    let headers: &HeaderMap = req.headers();
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    let decoded_bytes = decode(auth.clone()).expect("Failed to decode Base64 string");
    let decoded_credentials =
        std::str::from_utf8(&decoded_bytes).expect("Failed to convert bytes to string");
    let creds_vec: Vec<&str> = decoded_credentials.split(":").collect();
    let username = creds_vec.get(0).unwrap();
    let password = creds_vec.get(1).unwrap();
    if !creds.lock().unwrap().authenticate(username, password) {
        return HttpResponse::Unauthorized().json(ApiResponse::fail("Authentication failed"));
    }
    let result = cache.lock().unwrap().get_del(&cluster, &key);
    match result.value {
        Some(ref value) => HttpResponse::Ok().json(ApiResponse::ok(ResultValue {
            value: result.value,
            value_type: result.value_type,
        })),
        None => HttpResponse::NotFound().json(ApiResponse::fail("Key not found")),
    }
}
