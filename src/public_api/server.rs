
use actix_web::{web, App, HttpServer, HttpResponse, middleware::Logger};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use crate::cache::Cache;

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
}

pub async fn set(
    cache: web::Data<Arc<Mutex<Cache>>>,
    payload: web::Json<SetRequest>,
) -> HttpResponse {
    let SetRequest { cluster, key, value } = &*payload;
    let set_value = value.as_bytes();
    cache.lock().unwrap().set(cluster.clone(), key.clone(), Vec::from(set_value));
    HttpResponse::Ok().json(ApiResponse::ok("Set operation successful"))
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
    if clusters.len() > 0 {
        HttpResponse::Ok().json(ApiResponse::ok(clusters))
    } else {
        HttpResponse::Ok().json(ApiResponse::fail("No clusters found on this port"))
    }
}

pub async fn set_cluster(
    cache: web::Data<Arc<Mutex<Cache>>>,
    cluster: web::Path<String>,
) -> HttpResponse {
    let cluster_name = cluster.into_inner();
    cache.lock().unwrap().set_cluster(cluster_name);
    HttpResponse::Ok().json(ApiResponse::ok("Operation successful"))
}

pub async fn run_server(
    cache: Arc<Mutex<Cache>>,
    port_number: String,
    ip: String,
) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default()) // Enable request logging
            .app_data(web::Data::new(cache.clone()))
            .route("/api/set", web::post().to(set))
            .route("/api/get/{cluster}/{key}", web::get().to(get))
            .route("/api/delete/{cluster}/{key}", web::delete().to(delete))
            .route("/api/get_keys/{cluster}", web::get().to(get_keys_of_cluster))
            .route("/api/clear_cluster/{cluster}", web::delete().to(clear_cluster))
            .route("/api/get_clusters", web::get().to(get_all_clusters))
            .route("/api/set_cluster/{cluster}", web::post().to(set_cluster))
    })
    .bind(format!("{}:{}", ip, port_number))? // Bind to the provided IP and port
    .run()
    .await
}
