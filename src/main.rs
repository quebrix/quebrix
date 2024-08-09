use std::sync::{Arc, Mutex};
use crate::public_api::server;
use memory_handling::memory_handling::MemoryHandler;
mod cache;
mod public_api;
mod config;
mod memory_handling;
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
    
    // let args: Vec<String> = env::args().collect();
    // let main_args = args.last().unwrap();

    // Reading configurations from config.json
    let settings: Settings = Settings::new();
    let memory_handler = Arc::new(Mutex::new(MemoryHandler::new()));
    let cache = Arc::new(Mutex::new(Cache::new(settings.port, memory_handler.clone())));
    let cache_clone = Arc::clone(&cache);
    
    std::thread::spawn(move || {
        actix_web::rt::System::new().block_on(async move {
            let settings: Settings = Settings::new();
            server::run_server(cache_clone, settings.port.to_string(), "0.0.0.0".to_string()).await.unwrap();
        });
    });

    std::thread::park();

}