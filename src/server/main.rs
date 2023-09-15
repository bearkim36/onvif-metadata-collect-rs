/**
 * Copyright (C) 2022 bearkim forked by rertina 
 * 
 * 
 */
#[macro_use]
extern crate lazy_static;

use std::{thread, time, env};
use std::sync::mpsc::{self, Receiver, Sender};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Error};

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};

use tokio::task::JoinHandle;

mod server_metadata;
mod fclt;
mod util;

extern crate dotenv;

lazy_static! {
    static ref FCLT_DATA_MAP: Mutex<HashMap<String, server_metadata::MetadataConfig>> = Mutex::new(HashMap::new());
    
}
pub struct TaskManager {
    tasks: HashMap<String, Box<dyn ErasedJoinHandle>>,
}

trait ErasedJoinHandle {
    fn abort(&self);
}

impl<T> ErasedJoinHandle for JoinHandle<T> {
    fn abort(&self) {
        JoinHandle::abort(self);
    }
}
impl TaskManager {
    pub fn abort_all(&mut self) {
        for (_, handle) in self.tasks.drain() {
            handle.abort();
        }
    }

    pub fn abort(&mut self, key:String) {
        for (ckey, handle) in self.tasks.drain() {
            if ckey == key {
                handle.abort();
            }
        }
    }
    
    pub async fn add_task<T: 'static>(&mut self, key: String, handle: JoinHandle<T>) {        
        self.tasks.insert(key, Box::new(handle));
        
        //tokio::join!(handle);
    }
}

async fn server_mode(mut task_manager:TaskManager) -> Result<(), Error> {
    let mongo_uri = env::var("MONGO_URI").unwrap();
    let mongo_db_name = env::var("MONGO_DB_NAME").unwrap();
    let kafka_broker_1 = env::var("KAFKA_SERVER_BROKER_IP1").unwrap();
    
    println!("MONGO_URI: {}", mongo_uri);
    println!("MONGO_DB_NAME: {}", mongo_db_name);

    let server_ip = String::from(env::var("SERVER_ADDR").unwrap_or("0.0.0.0".to_string()));    
    let port : u16 = env::var("SERVER_PORT").unwrap_or("8000".to_string()).parse().unwrap();
    let fclt_obj = fclt::FcltLib::new(mongo_uri, mongo_db_name).await;
    let fclt_data = fclt_obj.get_fclt().await;

    let mut producer: &FutureProducer = &ClientConfig::new()
    .set("bootstrap.servers", kafka_broker_1)
    .set("message.timeout.ms", "5000")
    .set("security.protocol", "plaintext")
    .create()
    .expect("Producer creation error");
            
    let threads: Vec<_> = (0..fclt_data.len()).map(|i| {
        let fd = fclt_data.to_owned();
        let p = producer.to_owned();

        let rtsp_id = &fd[i].camera_id;
        let rtsp_pw = &fd[i].camera_pass;
        let fclt_id = &fd[i].fclt_id;
        let camera_ip = &fd[i].camera_ip;
        let http_port = &fd[i].http_port;
        let rtsp_port = &fd[i].rtsp_port;                

        let ai_cam_model = &fd[i].ai_cam_model;
        let img_save_path = String::from(env::var("IMG_SAVE_PATH").unwrap_or("./".to_string()));
        let face_recognition_url = String::from(env::var("FACE_RECOGNITION_URL").unwrap_or("".to_string()));

        let metadata_config = server_metadata::MetadataConfig { 
            camera_ip: String::from(camera_ip),
            http_port: String::from(http_port),
            rtsp_port: String::from(rtsp_port),                    
            username: String::from(rtsp_id), 
            password: String::from(rtsp_pw),
            fclt_id: String::from(fclt_id),
            img_save_path,
            ai_cam_model: String::from(ai_cam_model),
            face_recognition_url,
        };      

        let mut _fclt_data_map = FCLT_DATA_MAP.lock().unwrap();
        _fclt_data_map.insert(String::from(fclt_id), metadata_config.clone());

        let handle = metadata_thread(metadata_config, p);                
        
        task_manager.add_task(String::from(fclt_id), handle);
        

        // handle
    }).collect();

    // for handle in threads {
    //     tokio::join!(handle);        
    // }

    // let threads: Vec<_> = (0..fclt_data.len())
    //     .map(|i| {                       
    //         let fd = fclt_data.to_owned();
    //         let p = producer.to_owned();

            
    //         metadata_thread(&metadata, &p)            
    //         // tokio::spawn(async move {                                
    //         //     // let rtsp_url = env::var("RTSP_URL").unwrap();
    //         //     // let rtsp_id = env::var("RTSP_ID").unwrap();
    //         //     // let rtsp_pw = env::var("RTSP_PW").unwrap();
                
                        

    //         //     let mut manager = server_metadata::Manager {
    //         //         is_receiving: false
    //         //     };
    //         //     loop {                          

    //         //         if ai_cam_model.eq("") {
    //         //             break;                        
    //         //         }

    //         //         println!("{} Start ONVIF session", i);
    //         //         let _ = server_metadata::MetadataManager::run_onvif(&mut manager, metadata.clone(), p.clone()).await;
                    
    //         //         thread::sleep(time::Duration::from_secs(5));
    //         //     }
    //         // })
    //     })
    //     .collect();
    loop {
        thread::sleep(time::Duration::from_secs(5));
    }
    Ok(())
}


fn metadata_thread(metadata_config:server_metadata::MetadataConfig, p:FutureProducer) -> tokio::task::JoinHandle<()> {
    tokio::spawn( async move {        
        let mut manager = server_metadata::Manager {
            is_receiving: true
        };
        loop {                          

            if metadata_config.ai_cam_model.eq("") {
                break;                        
            }
            
            server_metadata::MetadataManager::run_onvif(&mut manager, metadata_config.clone(), p.clone()).await;
            
            thread::sleep(time::Duration::from_secs(5));
        }
    })
}


#[tokio::main]
async fn main() { 
    let path = util::get_env_path().expect("Couldn't");    
    println!("{}", path.display());
    dotenv::from_filename(path).expect("Failed to open directory");
    let mut task_manager = TaskManager{ tasks: HashMap::new() };

    println!("Boot on Server Mode");
    server_mode(task_manager).await.unwrap();            
    
}

