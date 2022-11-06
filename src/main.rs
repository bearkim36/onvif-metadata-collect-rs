/**
 * Copyright (C) 2022 bearkim forked by rertina 
 * 
 * 
 */

use std::{thread, time, env, io};
use std::path::PathBuf;
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};
use anyhow::{anyhow, Error};

mod edge_metadata;
mod server_metadata;
pub mod lpr;

extern crate dotenv;

async fn server_mode() -> Result<(), Error> {
    println!("MONGO_URI: {}", env::var("MONGO_URI").expect("MONGO_URI not found"));
    let mongo_uri = env::var("MONGO_URI").unwrap();
    let server_ip = String::from(env::var("SERVER_ADDR").unwrap_or("0.0.0.0".to_string()));
    let port : u16 = env::var("SERVER_PORT").unwrap_or("8000".to_string()).parse().unwrap();
    let client = Client::with_uri_str(mongo_uri).await.expect("failed to connect");
    let lpr_result = lpr::lpr_init();
    println!("LPR_RESULT: {:?}", lpr_result);

    tokio::spawn(async move { 
        loop {
            let result = lpr::anpr_read_file();
            println!("LPR_RESULT: {:?}", result);
            
        }
    }).await?
}

async fn edge_device_mode() -> Result<(), Error> {
    let rtsp_url = env::var("RTSP_URL").unwrap();
    let rtsp_id = env::var("RTSP_ID").unwrap();
    let rtsp_pw = env::var("RTSP_PW").unwrap();

    tokio::spawn(async move {                
        let metadata = edge_metadata::Metadata { 
            url: String::from(rtsp_url),
            username: String::from(rtsp_id), 
            password: String::from(rtsp_pw),
        };
        loop {
            println!("Start ONVIF session");
            if let Err(_err) = edge_metadata::MetadataManager::run_onvif(&metadata).await {
                println!("retry 5sec after");
                thread::sleep(time::Duration::from_secs(5));
            }
        }   
    }).await?
}

fn get_env_path() -> io::Result<PathBuf> {
    let mut dir = env::current_exe()?;
    dir.pop();    
    dir.push(".env");
    Ok(dir)
}


#[tokio::main]
async fn main() { 
    let path = get_env_path().expect("Couldn't");    
    println!("{}", path.display());
    dotenv::from_filename(path).expect("Failed to open directory");

    let mode: u16 = env::var("MODE").unwrap_or("1".to_string()).parse().unwrap();
    if mode == 1 {
        println!("Boot on Edge device Mode");
        edge_device_mode().await;        
    }
    else  {
        println!("Boot on Server Mode");
        server_mode().await;        
    }


}

