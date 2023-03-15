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
    let mongo_uri = env::var("MONGO_URI").unwrap();
    let mongo_db_name = env::var("MONGO_DB_NAME").unwrap();
    println!("MONGO_URI: {}", mongo_uri);
    println!("MONGO_DB_NAME: {}", mongo_db_name);

    let server_ip = String::from(env::var("SERVER_ADDR").unwrap_or("0.0.0.0".to_string()));    
    let port : u16 = env::var("SERVER_PORT").unwrap_or("8000".to_string()).parse().unwrap();
    let fclt_obj = fclt::FcltLib::new(mongo_uri, mongo_db_name).await;
    fclt_obj.get_fclt().await;

    let sample_list = [
        ["172.40.14.222",	  "admin",	"hanam2022!"],
    ];
    let threads: Vec<_> = (0..sample_list.len())
        .map(|i| {            
            tokio::spawn(async move {
                // let rtsp_url = env::var("RTSP_URL").unwrap();
                // let rtsp_id = env::var("RTSP_ID").unwrap();
                // let rtsp_pw = env::var("RTSP_PW").unwrap();
            
                let rtsp_url = sample_list[i][0];
                let rtsp_id = sample_list[i][1];
                let rtsp_pw = sample_list[i][2];
                let fclt_id = "test".to_string();
                let img_save_path = String::from(env::var("IMG_SAVE_PATH").unwrap_or("./".to_string()));

                let metadata = server_metadata::Metadata { 
                    url: String::from(rtsp_url),
                    username: String::from(rtsp_id), 
                    password: String::from(rtsp_pw),
                    fclt_id: String::from(fclt_id),
                    img_save_path: img_save_path,                    
                };
                loop {
                    println!("{} Start ONVIF session", i);
                    server_metadata::MetadataManager::run_onvif(&metadata).await;
                    
                    thread::sleep(time::Duration::from_secs(1));                    
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

