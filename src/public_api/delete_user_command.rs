use super::server::ApiResponse;
use super::server::SetRequest;
use super::server::UserRequest;
use crate::creds::auth::Authenticator;
use crate::creds::cred_manager::{CredsManager, RoleManagement, User};
use crate::creds::del_user::DeletUser;
use crate::creds::who_am_i::WhowAmI;
use actix_web::{
    http::header::HeaderMap, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer,
};
use base64::decode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub async fn delete_user(
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    input_username: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
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
    let mut cred = creds.lock().unwrap();
    let result = cred.delete_user(&input_username, None);
    if result.is_success {
        return HttpResponse::Ok().json(ApiResponse::ok(result.message));
    } else {
        return HttpResponse::Ok().json(ApiResponse::fail(result.message));
    }
}
