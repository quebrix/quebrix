use crate::{
    cache::{
        cache::ResultValue, clear_cluster::ClearCluster, decr::Decr, delete::Delete, get::Get,
        get_all_clusters::GetAllClusters, get_cluster_keys::GetClusterKeys, incr::Incr, set::Set,
        set_cluster::SetCluster, Cache,
    },
    creds::cred_manager::{CredsManager, RoleManagement, User},
    public_api::{
        acl_authenticate::authenticate_user, acl_set_user::add_user,
        check_connection_command::check_connection, clr_command::clear_cluster, decr_command::decr,
        del_command::delete, get_all_clusters_command::get_all_clusters, get_command::get,
        get_keys_command::get_keys_of_cluster, incr_command::incr,
        set_cluster_command::set_cluster, set_command::set,
    },
};
use actix_web::{
    http::header::HeaderMap, middleware::Logger, web, App, HttpRequest, HttpResponse, HttpServer,
};

use base64::decode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::get_del::get_del;

#[derive(Deserialize)]
pub struct UserRequest {
    pub username: String,
    pub password: String,
    pub role: String,
}

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub is_success: bool,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        ApiResponse {
            is_success: true,
            data,
        }
    }

    pub fn fail(data: T) -> Self {
        ApiResponse {
            is_success: false,
            data,
        }
    }
}

#[derive(Deserialize)]
pub struct SetRequest {
    pub cluster: String,
    pub key: String,
    pub value: String,
    pub ttl: Option<u64>, // Duration in milliseconds
}
#[derive(Deserialize)]

pub struct SetNumbericRequest {
    pub cluster: String,
    pub key: String,
    pub value: Option<i32>, //remember is i32
}

pub async fn run_server(
    cache: Arc<Mutex<Cache>>,
    creds: Arc<Mutex<CredsManager>>,
    port_number: String,
    ip: String,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default()) // Enable request logging
            .app_data(web::Data::new(cache.clone()))
            .app_data(web::Data::new(creds.clone()))
            .route("/api/set", web::post().to(set))
            .route("/api/incr", web::post().to(incr))
            .route("/api/decr", web::post().to(decr))
            .route("/api/get/{cluster}/{key}", web::get().to(get))
            .route("/api/get_del/{cluster}/{key}", web::get().to(get_del))
            .route("/api/ping", web::get().to(check_connection))
            .route("/api/delete/{cluster}/{key}", web::delete().to(delete))
            .route(
                "/api/get_keys/{cluster}",
                web::get().to(get_keys_of_cluster),
            )
            .route(
                "/api/clear_cluster/{cluster}",
                web::delete().to(clear_cluster),
            )
            .route("/api/get_clusters", web::get().to(get_all_clusters))
            .route("/api/set_cluster/{cluster}", web::post().to(set_cluster))
            .route("/api/add_user", web::post().to(add_user))
            .route("/api/login", web::post().to(authenticate_user))
    })
    .bind(format!("{}:{}", ip, port_number))? // Bind to the provided IP and port
    .run()
    .await
}
