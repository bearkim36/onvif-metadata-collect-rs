/**
 * Copyright (C) 2022 bearkim forked by rertina 
 * 
 * 
 */

use std::{thread, time, env, io};
use std::path::PathBuf;
use anyhow::{anyhow, Error};

mod edge_metadata;

extern crate dotenv;


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

    println!("Boot on Edge device Mode");
    edge_device_mode().await.unwrap();        
    
}

