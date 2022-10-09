// Copyright (C) 2022 bearkim forked by rertina

use std::{str::FromStr, thread::Thread};
use std::sync::Mutex;
use std::collections::HashMap;
use std::{thread, time};

use actix_web::{get, middleware, rt, web, App, HttpRequest, HttpServer, HttpResponse, Responder, cookie::time::Duration};
use lazy_static::lazy_static;

mod metadata;
use metadata::{Metadata, MetadataManager};


// #[tokio::main(flavor = "multi_thread", worker_threads = 1000)]
// async fn main() {    
    
//     let mut h = init_logging();        
//     let _a = h.async_scope();
//     for i in 1..20 {
//         tokio::spawn(async move {
//             run(i).await;    
//         });            
//     }
//     loop{
//         thread::sleep(time::Duration::from_millis(10));
//     }
// }
lazy_static! {
    static ref HASHMAP: Mutex<HashMap<u32, &'static str>> = {
        let mut m = HashMap::new();
        m.insert(0, "foo");
        m.insert(1, "bar");
        m.insert(2, "baz");
        Mutex::new(m)
    };        
}

#[get("/")]
async fn index(req: HttpRequest) -> &'static str {
    println!("REQ: {:?}", req);
    "Hello world!\r\n"
}

#[get("/addMetadataConnect")]
async fn add_metadata_connect() -> impl Responder {
  
    tokio::spawn(async move {                
        let metadata = Metadata { 
            url: String::from("rtsp://118.36.97.121/profile1/media.smp"),
            username: String::from("admin"), 
            password: String::from("!qaz2wsx"),
            // url: String::from("rtsp://192.168.0.7/profile1/media.smp"),
            // username: String::from("admin"), 
            // password: String::from("r00tr00tr00t"),
            fclt_name: String::from("test"),
        };
        loop {
            println!("start onvif session");
            if let Err(_err) = metadata.run_onvif().await {
                println!("retry 5sec after");
                thread::sleep(time::Duration::from_secs(5));
            }
        }   
    });            
    HttpResponse::Ok().body("added Meatadata Connection")
}

#[tokio::main]
async fn main()  -> std::io::Result<()> {    
    

    HttpServer::new(|| {
        App::new().service(index).service(add_metadata_connect)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

