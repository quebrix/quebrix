use actix_web::{http::header, middleware::Logger, web, App,HttpRequest, HttpResponse, HttpServer,http::header::HeaderMap};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::{cache::Cache, creds::cred_manager::CredsManager};
use std::time::Duration;


#[derive(Deserialize)]
struct UserRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct ApiResponse<T> {
    is_success: bool,
    data: T,
}

impl<T> ApiResponse<T> {
    fn ok(data: T) -> Self {
        ApiResponse {
            is_success: true,
            data,
        }
    }

    fn fail(data: T) -> Self {
        ApiResponse {
            is_success: false,
            data,
        }
    }
}

#[derive(Deserialize)]
struct SetRequest {
    cluster: String,
    key: String,
    value: String,
    ttl: Option<u64>, // Duration in milisecseconds
}

pub async fn add_user(
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    payload: web::Json<UserRequest>,
) -> HttpResponse {
    let UserRequest { username, password } = &*payload;

    match creds.lock().unwrap().add_user(username.clone(), password.clone()) {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::ok("User added successfully")),
        Err(err) => HttpResponse::InternalServerError().json(ApiResponse::fail(err.to_string())),
    }
}


pub async fn authenticate_user(
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    payload: web::Json<UserRequest>,
) -> HttpResponse {
    let UserRequest { username, password } = &*payload;

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

pub async fn set(
    cache: web::Data<Arc<Mutex<Cache>>>,
    payload: web::Json<SetRequest>,
    req: HttpRequest,
) -> HttpResponse {
    let SetRequest { cluster, key, value, ttl } = &*payload;
    let headers: &HeaderMap = req.headers();
    let username = headers.get("X-Username").and_then(|v| v.to_str().ok()).unwrap_or("");
    let password = headers.get("X-Password").and_then(|v| v.to_str().ok()).unwrap_or("");
    let set_value = value.as_bytes();
    let ttl_duration = ttl.map(|t| Duration::from_millis(t));
    let set_result = cache.lock().unwrap().set(cluster.clone(), key.clone(), Vec::from(set_value), ttl_duration,username,password);
    if !set_result{
        return HttpResponse::Ok().json(ApiResponse::fail("access denied login first"));
    }
    return HttpResponse::Ok().json(ApiResponse::ok("set successfully"));
}
pub async fn get(
    cache: web::Data<Arc<Mutex<Cache>>>,
    info: web::Path<(String, String)>,
) -> HttpResponse {
    let (cluster, key) = info.into_inner();
    match cache.lock().unwrap().get(&cluster, &key) {
        Some(value) => HttpResponse::Ok().json(ApiResponse::ok(value)),
        None => HttpResponse::NotFound().json(ApiResponse::fail("Key not found")),
    }
}

pub async fn delete(
    cache: web::Data<Arc<Mutex<Cache>>>,
    info: web::Path<(String, String)>,
) -> HttpResponse {
    let (cluster, key) = info.into_inner();
    cache.lock().unwrap().delete(&cluster, &key);
    HttpResponse::Ok().json(ApiResponse::ok("Delete operation successful"))
}

pub async fn clear_cluster(
    cache: web::Data<Arc<Mutex<Cache>>>,
    cluster: web::Path<String>,
) -> HttpResponse {
    cache.lock().unwrap().clear_cluster(&cluster);
    HttpResponse::Ok().json(ApiResponse::ok("Clear cluster operation successful"))
}

pub async fn get_keys_of_cluster(
    cache: web::Data<Arc<Mutex<Cache>>>,
    cluster: web::Path<String>,
) -> HttpResponse {
    let cluster_name = cluster.into_inner();
    let keys = cache.lock().unwrap().get_keys_of_cluster(&cluster_name);
    HttpResponse::Ok().json(ApiResponse::ok(keys))
}

pub async fn get_all_clusters(
    cache: web::Data<Arc<Mutex<Cache>>>,
) -> HttpResponse {
    let clusters = cache.lock().unwrap().get_all_clusters();
    if !clusters.is_empty() {
        HttpResponse::Ok().json(ApiResponse::ok(clusters))
    } else {
        HttpResponse::Ok().json(ApiResponse::fail("No clusters found on this port"))
    }
}

pub async fn check_connection() -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse::ok("Pong"))
}

pub async fn set_cluster(
    cache: web::Data<Arc<Mutex<Cache>>>,
    cluster: web::Path<String>,
) -> HttpResponse {
    let cluster_name = cluster.into_inner();
    cache.lock().unwrap().set_cluster(cluster_name);
    HttpResponse::Ok().json(ApiResponse::ok("Cluster set operation successful"))
}

pub async fn run_server(
    cache: Arc<Mutex<Cache>>,
    creds:Arc<Mutex<CredsManager>>,
    port_number: String,
    ip: String,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default()) // Enable request logging
            .app_data(web::Data::new(cache.clone()))
            .app_data(web::Data::new(creds.clone()))
            .route("/api/set", web::post().to(set))
            .route("/api/get/{cluster}/{key}", web::get().to(get))
            .route("/api/ping", web::get().to(check_connection))          
            .route("/api/delete/{cluster}/{key}", web::delete().to(delete))
            .route("/api/get_keys/{cluster}", web::get().to(get_keys_of_cluster))
            .route("/api/clear_cluster/{cluster}", web::delete().to(clear_cluster))
            .route("/api/get_clusters", web::get().to(get_all_clusters))
            .route("/api/set_cluster/{cluster}", web::post().to(set_cluster))
            .route("/api/add_profile", web::post().to(add_user))  
            .route("/api/login", web::post().to(authenticate_user))  
    })
    .bind(format!("{}:{}", ip, port_number))? // Bind to the provided IP and port
    .run()
    .await
}
