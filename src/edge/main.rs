/**
 * Copyright (C) 2022 bearkim forked by rertina 
 * 
 * 
 */
use std::{thread, time, env};
use anyhow::Error;

mod edge_metadata;

async fn edge_device_mode(rtsp_url:String, rtsp_id:String, rtsp_pw:String, analysis_url:String) -> Result<(), Error> {    
    tokio::spawn(async move {
        let metadata = edge_metadata::Metadata { 
            url: String::from(rtsp_url),
            username: String::from(rtsp_id), 
            password: String::from(rtsp_pw),
        };
        loop {
            println!("Start ONVIF session");
            if let Err(_err) = edge_metadata::MetadataManager::run_onvif(&metadata, analysis_url.clone()).await {
                println!("retry 5sec after");
                thread::sleep(time::Duration::from_secs(5));
            }
        }   
    }).await?
}



#[tokio::main]
async fn main() { 
    let args: Vec<String> = env::args().collect();        
    println!("RTSP URL: {}", args[1]);
    println!("ADMIN: {}", args[2]);
    println!("PASSWORD: {}", args[3]);
    println!("ANALYSIS_URL: {}", args[4]);
    println!("Boot on Edge device Mode");
    edge_device_mode(args[1].clone(), args[2].clone(), args[3].clone(), args[4].clone()).await.unwrap();        
    
}

