/**
 * Copyright (C) 2022 bearkim forked by rertina 
 * 
 * 
 */

use std::{thread, time, env};
use anyhow::{anyhow, Error};

mod server_metadata;
mod fclt;
mod util;

extern crate dotenv;

async fn server_mode() -> Result<(), Error> {
    let mongo_uri = env::var("MONGO_URI").unwrap();
    let mongo_db_name = env::var("MONGO_DB_NAME").unwrap();
    println!("MONGO_URI: {}", mongo_uri);
    println!("MONGO_DB_NAME: {}", mongo_db_name);

    let server_ip = String::from(env::var("SERVER_ADDR").unwrap_or("0.0.0.0".to_string()));    
    let port : u16 = env::var("SERVER_PORT").unwrap_or("8000".to_string()).parse().unwrap();
    let fclt_obj = fclt::FcltLib::new(mongo_uri, mongo_db_name).await;
    let fclt_data = fclt_obj.get_fclt().await;


    let threads: Vec<_> = (0..fclt_data.len())
        .map(|i| {                       
            let fd = fclt_data.to_owned();
            tokio::spawn(async move {                                
                // let rtsp_url = env::var("RTSP_URL").unwrap();
                // let rtsp_id = env::var("RTSP_ID").unwrap();
                // let rtsp_pw = env::var("RTSP_PW").unwrap();
                
                let rtsp_id = &fd[i].camera_id;
                let rtsp_pw = &fd[i].camera_pass;
                let fclt_id = &fd[i].fclt_id;
                let camera_ip = &fd[i].camera_ip;
                let http_port = &fd[i].http_port;
                let rtsp_port = &fd[i].rtsp_port;                

                let ai_cam_model = &fd[i].ai_cam_model;
                let img_save_path = String::from(env::var("IMG_SAVE_PATH").unwrap_or("./".to_string()));
                let face_recognition_url = String::from(env::var("FACE_RECOGNITION_URL").unwrap_or("".to_string()));

                let metadata = server_metadata::MetadataConfig { 
                    camera_ip: String::from(camera_ip),
                    http_port: String::from(http_port),
                    rtsp_port: String::from(rtsp_port),                    
                    username: String::from(rtsp_id), 
                    password: String::from(rtsp_pw),
                    fclt_id: String::from(fclt_id),
                    img_save_path,
                    ai_cam_model: String::from(ai_cam_model),
                    face_recognition_url
                };                
                loop {                          

                    if ai_cam_model.eq("") {
                        break;                        
                    }

                    println!("{} Start ONVIF session", i);
                    server_metadata::MetadataManager::run_onvif(&metadata).await;
                    
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


#[tokio::main]
async fn main() { 
    let path = util::get_env_path().expect("Couldn't");    
    println!("{}", path.display());
    dotenv::from_filename(path).expect("Failed to open directory");

    println!("Boot on Server Mode");
    server_mode().await;        

}

