use super::server::ApiResponse;
use super::server::KeysCountRequest;
use super::server::SetRequest;
use super::server::TypeOfKeyRequest;
use super::server::UserRequest;
use crate::cache::exist_key::KeyExists;
use crate::cache::keys_count::KeysCount;
use crate::cache::type_of_key::TypeOfKey;
use crate::creds::auth::Authenticator;
use crate::{
    cache::{
        cache::ResultValue, clear_cluster::ClearCluster, decr::Decr, delete::Delete, get::Get,
        get_all_clusters::GetAllClusters, get_cluster_keys::GetClusterKeys, incr::Incr, set::Set,
        set_cluster::SetCluster, Cache,
    },
    creds::cred_manager::{CredsManager, RoleManagement, User},
};
use actix_web::{
    http::header::HeaderMap, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer,
};
use base64::decode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;
pub async fn keys_count(
    cache: web::Data<Arc<Mutex<Cache>>>,
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    payload: web::Json<KeysCountRequest>,
    req: HttpRequest,
) -> HttpResponse {
    let KeysCountRequest { cluster } = &*payload;
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
    let set_result = cache.lock().unwrap().keys_count(cluster.as_str());

    if set_result != 0 {
        HttpResponse::Ok().json(ApiResponse::ok(set_result))
    } else {
        HttpResponse::Ok().json(ApiResponse::fail(0))
    }
}
