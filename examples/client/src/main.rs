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
use dkcalculation::{ DKCalculation, DKFcltInstallInfo};

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
    println!("SERVER_ADDR: {}", env::var("SERVER_ADDR").unwrap_or("0.0.0.0".to_string()));
    println!("SERVER_PORT: {}", env::var("SERVER_PORT").unwrap_or("8000".to_string()));
    let mongo_uri = env::var("MONGO_URI").unwrap();
    let server_ip = String::from(env::var("SERVER_ADDR").unwrap_or("0.0.0.0".to_string()));
    let port : u16 = env::var("SERVER_PORT").unwrap_or("8000".to_string()).parse().unwrap();

    let client = Client::with_uri_str(mongo_uri).await.expect("failed to connect");
    
    let dk_fclt_install_info = DKFcltInstallInfo{
        view_angle_w : env::var("VIEW_ANGLE_W").unwrap_or("36.0".to_string()).parse::<f64>().unwrap(),
        lens_distance: env::var("LENS_DISTANCE").unwrap_or("5.0".to_string()).parse::<f64>().unwrap(),
        camera_screen_width: env::var("CAMERA_SCREEN_WIDTH").unwrap_or("4.0".to_string()).parse::<f64>().unwrap(),
        installation_height: env::var("INSTALLATION_HEIGHT").unwrap_or("6.0".to_string()).parse::<f64>().unwrap(),
    };

    //let dkcalc = DKCalculation::new(dk_fclt_install_info);
    

    


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

