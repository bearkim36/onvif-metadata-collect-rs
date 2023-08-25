/**
 * Copyright (C) 2022 bearkim forked by rertina 
 * 
 * 
 */
use std::{thread, time, env};
use clap::Parser;
use anyhow::Error;

mod edge_metadata;

#[derive(Parser)]
struct Opts {
    #[arg(long)]
    url: Option<String>,

    #[arg(long)]
    username: Option<String>,

    #[arg(long, requires = "username")]
    password: Option<String>,
    
    #[arg( long)]
    analysis: Option<String>,    
}

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
    let opts = Opts::parse();

    let rtsp_url = opts.url.as_deref().unwrap().to_string();
    let rtsp_id = opts.username.as_deref().unwrap().to_string();
    let rtsp_pw = opts.password.as_deref().unwrap().to_string();
    let analysis_url = opts.analysis.as_deref().unwrap().to_string();
    println!("RTSP URL: {}", rtsp_url);
    println!("ADMIN: {}", rtsp_id);
    println!("PASSWORD: {}", rtsp_pw);
    println!("ANALYSIS_URL: {}", analysis_url);
    println!("Boot on Edge device Mode");

    edge_device_mode(rtsp_url, rtsp_id, rtsp_pw, analysis_url).await.unwrap();        
    
}

