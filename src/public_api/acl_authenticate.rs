use super::server::ApiResponse;
use super::server::UserRequest;
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

pub async fn authenticate_user(
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    payload: web::Json<UserRequest>,
) -> HttpResponse {
    let UserRequest {
        username,
        password,
        role: _,
    } = &*payload;

    if creds.lock().unwrap().authenticate(username, password) {
        HttpResponse::Ok().json(ApiResponse {
            is_success: true,
            data: "Authentication successful".to_string(),
        })
    } else {
        HttpResponse::Unauthorized().json(ApiResponse {
            is_success: false,
            data: "Authentication failed".to_string(),
        })
    }
}
