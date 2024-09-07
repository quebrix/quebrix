use super::server::ApiResponse;
use super::server::SetRequest;
use crate::cache::mget::MGet;
use crate::{
    cache::{set::Set, Cache},
    creds::cred_manager::CredsManager,
};
use actix_web::{http::header::HeaderMap, web, HttpRequest, HttpResponse};
use base64::decode;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
pub async fn mget(
    cache: web::Data<Arc<Mutex<Cache>>>,
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    payload: web::Json<HashMap<String, Vec<String>>>,
    req: HttpRequest,
) -> HttpResponse {
    let cluster_keys = &*payload;
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

    let result = cache.lock().unwrap().mget(cluster_keys.clone());

    HttpResponse::Ok().json(ApiResponse::ok(result))
}
