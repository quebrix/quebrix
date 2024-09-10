use crate::{
    cache::{
        cache::ResultValue, clear_cluster::ClearCluster, decr::Decr, delete::Delete, get::Get,
        get_all_clusters::GetAllClusters, get_cluster_keys::GetClusterKeys, incr::Incr, set::Set,
        set_cluster::SetCluster, Cache,
    },
    creds::{
        cred_manager::{CredsManager, RoleManagement, User},
        load_user_from_file,
    },
    public_api::{
        acl_authenticate::authenticate_user, acl_set_user::add_user,
        check_connection_command::check_connection, clr_command::clear_cluster, decr_command::decr,
        del_command::delete, get_all_clusters_command::get_all_clusters, get_command::get,
        get_keys_command::get_keys_of_cluster, incr_command::incr, load_users_command::load_users,
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

use super::{
    delete_user_command::delete_user, load_users_from_file_command::load_users_from_file,
    who_am_i_command::who_am_i,
};

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
            .route("/api/load_users", web::get().to(load_users))
            .route(
                "/api/load_users_from_file",
                web::post().to(load_users_from_file),
            )
            .route("/api/who_am_i", web::get().to(who_am_i))
            .route("/api/incr", web::post().to(incr))
            .route("/api/decr", web::post().to(decr))
            .route("/api/get/{cluster}/{key}", web::get().to(get))
            .route("/api/delete_user/{username}", web::delete().to(delete_user))
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
