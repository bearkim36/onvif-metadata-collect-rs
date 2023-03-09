/**
 * Copyright (C) 2022 bearkim forked by rertina 
 * 
 * 
 */

use std::{thread, time, env, io};
use std::path::PathBuf;
use anyhow::{anyhow, Error};

mod server_metadata;
mod fclt;

extern crate dotenv;

async fn server_mode() -> Result<(), Error> {
    println!("MONGO_URI: {}", env::var("MONGO_URI").expect("MONGO_URI not found"));
    let mongo_uri = env::var("MONGO_URI").unwrap();
    let mongo_db_name = env::var("MONGO_DB_NAME").unwrap();
    let server_ip = String::from(env::var("SERVER_ADDR").unwrap_or("0.0.0.0".to_string()));
    let port : u16 = env::var("SERVER_PORT").unwrap_or("8000".to_string()).parse().unwrap();
    let fclt = fclt::FcltLib::new(mongo_uri, mongo_db_name);
    fclt.await.get_fclt();

    // #[cfg(target_os = "windows")]
    // let lpr_result = server_metadata::lpr::lpr_init();

    let threads: Vec<_> = (0..1)
        .map(|i| {            
            tokio::spawn(async move {
                let rtsp_url = env::var("RTSP_URL").unwrap();
                let rtsp_id = env::var("RTSP_ID").unwrap();
                let rtsp_pw = env::var("RTSP_PW").unwrap();
            
                let metadata = server_metadata::Metadata { 
                    url: String::from(rtsp_url),
                    username: String::from(rtsp_id), 
                    password: String::from(rtsp_pw),
                };
                loop {
                    println!("{} Start ONVIF session", i);
                    server_metadata::MetadataManager::run_onvif(&metadata).await;
                    
                    println!("retry 5sec after");
                    thread::sleep(time::Duration::from_secs(5));                    
                }
            })
        })
        .collect();

    for handle in threads {
        tokio::join!(handle);
    //     handle.join().unwrap();
    }

    Ok(())
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

