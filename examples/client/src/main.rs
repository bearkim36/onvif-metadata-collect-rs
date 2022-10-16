// Copyright (C) 2022 bearkim forked by rertina

use std::sync::Mutex;
use std::collections::HashMap;
use std::{thread, time, env, io};
use std::path::PathBuf;
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};

use actix_web::{get, middleware, rt, web, App, HttpRequest, HttpServer, HttpResponse, Responder, cookie::time::Duration};
use lazy_static::lazy_static;

mod metadata;
use metadata::{Metadata, MetadataManager};

mod dkcalculation;
use dkcalculation::{ DKCalculation};

extern crate dotenv;
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

fn get_env_path() -> io::Result<PathBuf> {
    let mut dir = env::current_exe()?;
    dir.pop();    
    dir.push(".env");
    Ok(dir)
}


#[tokio::main]
async fn main()  -> std::io::Result<()> { 
    let path = get_env_path().expect("Couldn't");    
    println!("{}", path.display());
    dotenv::from_filename(path).expect("Failed to open directory");

    println!("MONGO_URI: {}", env::var("MONGO_URI").expect("MONGO_URI not found"));
    println!("SERVER_ADDR: {}", env::var("SERVER_ADDR").expect("SERVER_ADDR not found"));             
    println!("SERVER_PORT: {}", env::var("SERVER_PORT").expect("SERVER_PORT not found"));             
    let mongo_uri = std::env::var("MONGO_URI").unwrap();
    let server_ip = String::from(env::var("SERVER_ADDR").unwrap());
    let port : u16 = env::var("SERVER_PORT").expect("is not valid").parse().unwrap();

    let client = Client::with_uri_str(mongo_uri).await.expect("failed to connect");

    let dkcalc = DKCalculation{..Default::default()};
    let calc = dkcalc.get_values();

    HttpServer::new(move || {
        App::new()
        .app_data(web::Data::new(client.clone()))
        .service(index)
        .service(add_metadata_connect)
    })
    .bind((server_ip, port))?
    .run()
    .await
}

