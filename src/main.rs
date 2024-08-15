use std::sync::{Arc, Mutex};
use crate::public_api::server;
use memory_handling::memory_handling::MemoryHandler;
mod cache;
use crate::logger::logger_manager::Logger;
mod public_api;
mod config;
mod memory_handling;
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
        println!("invalid EvictionStrategy passing in config.json change it in to => 0:VolatileLru 1:VolatileTtl 2:AllKeysLru 3:AllKeysRandom");
    }
    else {
        let mut log = Logger::log_info("application starting...");
        log.write_log_to_file();
        let memory_handler = Arc::new(Mutex::new(MemoryHandler::new()));
        let cache = Arc::new(Mutex::new(Cache::new(settings.port, memory_handler.clone(),settings.eviction_strategy)));
        let cache_clone = Arc::clone(&cache);
        
        std::thread::spawn(move || {
            actix_web::rt::System::new().block_on(async move {
                let settings: Settings = Settings::new();
                server::run_server(cache_clone, settings.port.to_string(), "0.0.0.0".to_string()).await.unwrap();
            });
        });
    
        std::thread::park();
       
    }
}