/**
 * Copyright (C) 2022 bearkim forked by rertina 
 * 
 * 
 */

use std::{thread, time, env, io};
use std::path::PathBuf;
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};
use anyhow::{anyhow, Error};

mod server_metadata;

extern crate dotenv;

async fn server_mode() -> Result<(), Error> {
    println!("MONGO_URI: {}", env::var("MONGO_URI").expect("MONGO_URI not found"));
    let mongo_uri = env::var("MONGO_URI").unwrap();
    let server_ip = String::from(env::var("SERVER_ADDR").unwrap_or("0.0.0.0".to_string()));
    let port : u16 = env::var("SERVER_PORT").unwrap_or("8000".to_string()).parse().unwrap();
    let client = Client::with_uri_str(mongo_uri).await.expect("failed to connect");
    let lpr_result = server_metadata::lpr::lpr_init();
    println!("LPR_RESULT: {:?}", lpr_result);

    tokio::spawn(async move { 
        loop {
           
            
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

    println!("Boot on Server Mode");
    server_mode().await;        


}

