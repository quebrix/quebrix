
use actix_web::{
    http::{header::HeaderMap},
    middleware::Logger,
    web, App, HttpRequest, HttpResponse, HttpServer,
};
use std::str::FromStr;
use std::fmt;
use serde::{Deserialize, Serialize};
use tokio::runtime::TryCurrentError;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use base64::decode;
use crate::{cache::Cache, creds::cred_manager::{CredsManager, RoleManagement,User}};

#[derive(Deserialize)]
struct UserRequest {
    username: String,
    password: String,
    role: String,
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
    ttl: Option<u64>, // Duration in milliseconds
}

pub async fn add_user(
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    payload: web::Json<UserRequest>,
    req: HttpRequest,
) -> HttpResponse {
    let headers: &HeaderMap = req.headers();
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    let decoded_bytes = decode(auth.clone()).expect("Failed to decode Base64 string");
    let decoded_credentials = std::str::from_utf8(&decoded_bytes).expect("Failed to convert bytes to string");
    let creds_vec: Vec<&str> = decoded_credentials.split(":").collect();
    let username = creds_vec.get(0).unwrap();
    let auth_result = creds.lock().unwrap().authenticate(username, creds_vec.get(1).unwrap());
    let mut current_user:User;
    if auth_result{
       current_user = creds.lock().unwrap().get_user(username);
    }
    else{
        return HttpResponse::Ok().json(ApiResponse::fail("invalid pass or username"));
    }

    if !creds.lock().unwrap().is_admin(&current_user) {
        return HttpResponse::Ok().json(ApiResponse::fail("Permission denied: Admin role required to add users"));
    }

    let UserRequest { username, password, role } = &*payload;

    match creds.lock().unwrap().add_user(username.clone(), password.clone(), role.clone().parse::<RoleManagement>().unwrap(), Option::Some((&current_user))) {
        Ok(_) => HttpResponse::Ok().json(ApiResponse::ok("User added successfully")),
        Err(err) => HttpResponse::InternalServerError().json(ApiResponse::fail(err.to_string())),
    }
}

pub async fn authenticate_user(
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    payload: web::Json<UserRequest>,
) -> HttpResponse {
    let UserRequest { username, password, role:_ } = &*payload;

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
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    payload: web::Json<SetRequest>,
    req: HttpRequest,
) -> HttpResponse {
    let SetRequest { cluster, key, value, ttl } = &*payload;
    let headers: &HeaderMap = req.headers();
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    let decoded_bytes = decode(auth.clone()).expect("Failed to decode Base64 string");
    let decoded_credentials = std::str::from_utf8(&decoded_bytes).expect("Failed to convert bytes to string");
    let creds_vec: Vec<&str> = decoded_credentials.split(":").collect();
    let username = creds_vec.get(0).unwrap();
    let password = creds_vec.get(1).unwrap();

    if !creds.lock().unwrap().authenticate(username, password) {
        return HttpResponse::Unauthorized().json(ApiResponse::fail("Authentication failed"));
    }

    let set_value = value.as_bytes();
    let ttl_duration = ttl.map(Duration::from_millis);
    let set_result = cache.lock().unwrap().set(cluster.clone(), key.clone(), Vec::from(set_value), ttl_duration, username, password);
    
    if set_result {
        HttpResponse::Ok().json(ApiResponse::ok("Set operation successful"))
    } else {
        HttpResponse::Ok().json(ApiResponse::fail("Set operation failed"))
    }
}

pub async fn get(
    cache: web::Data<Arc<Mutex<Cache>>>,
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    info: web::Path<(String, String)>,
    req: HttpRequest,
) -> HttpResponse {
    let (cluster, key) = info.into_inner();
    let headers: &HeaderMap = req.headers();
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    let decoded_bytes = decode(auth.clone()).expect("Failed to decode Base64 string");
    let decoded_credentials = std::str::from_utf8(&decoded_bytes).expect("Failed to convert bytes to string");
    let creds_vec: Vec<&str> = decoded_credentials.split(":").collect();
    let username = creds_vec.get(0).unwrap();
    let password = creds_vec.get(1).unwrap();
    if !creds.lock().unwrap().authenticate(username, password) {
        return HttpResponse::Unauthorized().json(ApiResponse::fail("Authentication failed"));
    }
    match cache.lock().unwrap().get(&cluster, &key) {
        Some(value) => HttpResponse::Ok().json(ApiResponse::ok(value)),
        None => HttpResponse::NotFound().json(ApiResponse::fail("Key not found")),
    }
}

pub async fn delete(
    cache: web::Data<Arc<Mutex<Cache>>>,
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    info: web::Path<(String, String)>,
    req: HttpRequest,
) -> HttpResponse {
    let (cluster, key) = info.into_inner();
    let headers: &HeaderMap = req.headers();
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    let decoded_bytes = decode(auth.clone()).expect("Failed to decode Base64 string");
    let decoded_credentials = std::str::from_utf8(&decoded_bytes).expect("Failed to convert bytes to string");
    let creds_vec: Vec<&str> = decoded_credentials.split(":").collect();
    let username = creds_vec.get(0).unwrap();
    let password = creds_vec.get(1).unwrap();

    if !creds.lock().unwrap().authenticate(username, password) {
        return HttpResponse::Unauthorized().json(ApiResponse::fail("Authentication failed"));
    }
    cache.lock().unwrap().delete(&cluster, &key);
    HttpResponse::Ok().json(ApiResponse::ok("Delete operation successful"))
}

pub async fn clear_cluster(
    cache: web::Data<Arc<Mutex<Cache>>>,
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    cluster: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let headers: &HeaderMap = req.headers();
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    let decoded_bytes = decode(auth.clone()).expect("Failed to decode Base64 string");
    let decoded_credentials = std::str::from_utf8(&decoded_bytes).expect("Failed to convert bytes to string");
    let creds_vec: Vec<&str> = decoded_credentials.split(":").collect();
    let username = creds_vec.get(0).unwrap();
    let password = creds_vec.get(1).unwrap();

    if !creds.lock().unwrap().authenticate(username, password) {
        return HttpResponse::Unauthorized().json(ApiResponse::fail("Authentication failed"));
    }
    cache.lock().unwrap().clear_cluster(&cluster);
    HttpResponse::Ok().json(ApiResponse::ok("Clear cluster operation successful"))
}

pub async fn get_keys_of_cluster(
    cache: web::Data<Arc<Mutex<Cache>>>,
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    cluster: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let cluster_name = cluster.into_inner();
    let headers: &HeaderMap = req.headers();
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    let decoded_bytes = decode(auth.clone()).expect("Failed to decode Base64 string");
    let decoded_credentials = std::str::from_utf8(&decoded_bytes).expect("Failed to convert bytes to string");
    let creds_vec: Vec<&str> = decoded_credentials.split(":").collect();
    let username = creds_vec.get(0).unwrap();
    let password = creds_vec.get(1).unwrap();

    if !creds.lock().unwrap().authenticate(username, password) {
        return HttpResponse::Unauthorized().json(ApiResponse::fail("Authentication failed"));
    }
    let keys = cache.lock().unwrap().get_keys_of_cluster(&cluster_name);
    HttpResponse::Ok().json(ApiResponse::ok(keys))
}

pub async fn get_all_clusters(
    cache: web::Data<Arc<Mutex<Cache>>>,
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    req: HttpRequest,
) -> HttpResponse {
    let clusters = cache.lock().unwrap().get_all_clusters();
    let headers: &HeaderMap = req.headers();
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    let decoded_bytes = decode(auth.clone()).expect("Failed to decode Base64 string");
    let decoded_credentials = std::str::from_utf8(&decoded_bytes).expect("Failed to convert bytes to string");
    let creds_vec: Vec<&str> = decoded_credentials.split(":").collect();
    let username = creds_vec.get(0).unwrap();
    let password = creds_vec.get(1).unwrap();

    if !creds.lock().unwrap().authenticate(username, password) {
        return HttpResponse::Unauthorized().json(ApiResponse::fail("Authentication failed"));
    }
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
    creds: web::Data<Arc<Mutex<CredsManager>>>,
    cache: web::Data<Arc<Mutex<Cache>>>,
    cluster: web::Path<String>,
    req: HttpRequest,
) -> HttpResponse {
    let cluster_name = cluster.into_inner();
    let headers: &HeaderMap = req.headers();
    let auth = headers.get("Authorization").unwrap().to_str().unwrap();
    let decoded_bytes = decode(auth.clone()).expect("Failed to decode Base64 string");
    let decoded_credentials = std::str::from_utf8(&decoded_bytes).expect("Failed to convert bytes to string");
    let creds_vec: Vec<&str> = decoded_credentials.split(":").collect();
    let username = creds_vec.get(0).unwrap();
    let password = creds_vec.get(1).unwrap();

    if !creds.lock().unwrap().authenticate(username, password) {
        return HttpResponse::Unauthorized().json(ApiResponse::fail("Authentication failed"));
    }
    cache.lock().unwrap().set_cluster(cluster_name);
    HttpResponse::Ok().json(ApiResponse::ok("Cluster set operation successful"))
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
            .route("/api/get/{cluster}/{key}", web::get().to(get))
            .route("/api/ping", web::get().to(check_connection))
            .route("/api/delete/{cluster}/{key}", web::delete().to(delete))
            .route("/api/get_keys/{cluster}", web::get().to(get_keys_of_cluster))
            .route("/api/clear_cluster/{cluster}", web::delete().to(clear_cluster))
            .route("/api/get_clusters", web::get().to(get_all_clusters))
            .route("/api/set_cluster/{cluster}", web::post().to(set_cluster))
            .route("/api/add_user", web::post().to(add_user))  
            .route("/api/login", web::post().to(authenticate_user))  
    })
    .bind(format!("{}:{}", ip, port_number))? // Bind to the provided IP and port
    .run()
    .await
}
