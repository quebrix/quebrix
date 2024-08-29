use std::sync::{Arc, Mutex};
use crate::public_api::server;
use memory_handling::memory_handling::MemoryHandler;
mod cache;
mod crypto;
mod creds;
mod persistent;
use crate::logger::logger_manager::Logger;
mod public_api;
mod config;
mod memory_handling;
use creds::cred_manager::CredsManager;
mod logger;
use std::env;
use crate::config::Settings;
use cache::Cache;

fn main() {
    println!("
    ");
    println!("|============================================================================|");
    println!("||  //////////  //     //   /////////   /////////    /////////    //        ||");
    println!("||  //      //  //     //   //     /    //     /     //           //        ||");
    println!("||  //     //   //     //   //          //           //           //        ||");
    println!("||  ////////    //     //   /////////   /////////    /////////    //        ||");
    println!("||  //     //   //     //          //          //    //           //        ||");
    println!("||  //     //   //     //   /     //    /     //     //           //        ||");
    println!("||  //     //   ////////   /////////   /////////     /////////    ///////// ||");
    println!("|============================================================================|");
    println!("
    ");
    
    
    // Reading configurations from config.json
    let settings: Settings = Settings::new();
    if settings.eviction_strategy > 3 {
        println!("invalid EvictionStrategy passing in config.json change it in to => 0:VolatileLru 1:VolatileTtl 2:AllKeysLru 3:AllKeysRandom, check logs to more info.");
        let  evic_log = Logger::log_error("invalid EvictionStrategy passing in config.json change it in to => 0:VolatileLru 1:VolatileTtl 2:AllKeysLru 3:AllKeysRandom");
        evic_log.write_log_to_file();
    }
    else {
        let  log = Logger::log_info("application starting...");
        log.write_log_to_file();
        let memory_handler = Arc::new(Mutex::new(MemoryHandler::new()));
        let cred_manager = Arc::new(Mutex::new(CredsManager::new(settings.enable_logger)));
        let  memory_log = Logger::log_info("access to memory handling ...");
        memory_log.write_log_to_file();
        let cache = Arc::new(Mutex::new(Cache::new(settings.port, memory_handler.clone(),settings.eviction_strategy,settings.enable_logger,settings.persistent,cred_manager.clone())));
        let cache_clone = Arc::clone(&cache);
        let cred_clone = Arc::clone(&cred_manager);
        let  cache_log = Logger::log_info("cahce successfully instaled ...");
        cache_log.write_log_to_file();
        std::thread::spawn(move || {
            actix_web::rt::System::new().block_on(async move {
                let settings: Settings = Settings::new();
                server::run_server(cache_clone,cred_clone, settings.port.to_string(), "0.0.0.0".to_string()).await.unwrap();
            });
        });
        let  server_log = Logger::log_info("server setup success");
        server_log.write_log_to_file();
    
        let  ready_log = Logger::log_info("cache ready to use");
        ready_log.write_log_to_file();
        std::thread::park();
       
    }
}