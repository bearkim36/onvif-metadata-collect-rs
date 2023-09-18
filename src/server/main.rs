/**
 * Copyright (C) 2022 bearkim forked by rertina 
 * 
 * 
 */
#[macro_use]
extern crate lazy_static;

use async_once::AsyncOnce;
use fclt::FcltData;

use std::{thread, time, env};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Error};

use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::message::{Headers, Message};
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::get_rdkafka_version;

use tokio::task::JoinHandle;

use actix_web::middleware::Logger;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;


mod server_metadata;
mod fclt;
mod util;

extern crate dotenv;

lazy_static! {
    static ref FCLT_DATA_MAP: Mutex<HashMap<String, server_metadata::MetadataConfig>> = Mutex::new(HashMap::new());
    static ref TASK_MAP: Mutex<HashMap<String,JoinHandle<()>>> =  Mutex::new(HashMap::new());
    static ref PRODUCER: AsyncOnce<FutureProducer> = AsyncOnce::new(async {        
            let kafka_broker_1 = env::var("KAFKA_SERVER_BROKER_IP1").unwrap();                
            ClientConfig::new()
            .set("bootstrap.servers", kafka_broker_1)
            .set("message.timeout.ms", "5000")
            .set("security.protocol", "plaintext")
            .create()
            .expect("Producer creation error")        
    });
}


#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

async fn server_mode() -> Result<(), Error> {
    let mongo_uri = env::var("MONGO_URI").unwrap();
    let mongo_db_name = env::var("MONGO_DB_NAME").unwrap();    
    let fclt_obj = fclt::FcltLib::new(mongo_uri, mongo_db_name).await;
    let fclt_data = fclt_obj.get_fclt().await;
    
    for fd in fclt_data.clone().into_iter() {        
        thread_proc(fd);        
    }    

    Ok(())
}


fn thread_proc(fd:FcltData) {

    let rtsp_id = fd.camera_id;
    let rtsp_pw = fd.camera_pass;
    let fclt_id = fd.fclt_id;
    let camera_ip = fd.camera_ip;
    let http_port = fd.http_port;
    let rtsp_port = fd.rtsp_port;                

    let ai_cam_model = fd.ai_cam_model;
    let img_save_path = String::from(env::var("IMG_SAVE_PATH").unwrap_or("./".to_string()));
    let face_recognition_url = String::from(env::var("FACE_RECOGNITION_URL").unwrap_or("".to_string()));

    let metadata_config = server_metadata::MetadataConfig { 
        camera_ip: String::from(camera_ip),
        http_port: String::from(http_port),
        rtsp_port: String::from(rtsp_port),                    
        username: String::from(rtsp_id), 
        password: String::from(rtsp_pw),
        fclt_id: String::from(fclt_id.clone()),
        img_save_path,
        ai_cam_model: String::from(ai_cam_model),
        face_recognition_url,
    };      
    
    let mut _fclt_data_map = FCLT_DATA_MAP.lock().unwrap();
    if _fclt_data_map.contains_key(&fclt_id) {
        _fclt_data_map.remove(&fclt_id);
    }
    _fclt_data_map.insert(String::from(fclt_id.clone()), metadata_config.clone());

    let handle = tokio::spawn( async move {        
        let mut manager = server_metadata::Manager {
            is_receiving: true
        };
        loop {                          

            if metadata_config.ai_cam_model.eq("") {
                break;                        
            }
            let p = PRODUCER.get().await;
            server_metadata::MetadataManager::run_onvif(&mut manager, metadata_config.clone(), p.clone()).await;
            
            thread::sleep(time::Duration::from_secs(5));
        }
    });
    let mut _task_map = TASK_MAP.lock().unwrap();
    if _task_map.contains_key(&fclt_id) {
        if let Some(task_handler) = _task_map.get(&fclt_id) {
            task_handler.abort();
        }
        _task_map.remove(&fclt_id);
    }
    _task_map.insert(String::from(fclt_id), handle);                   
}
/*
async fn kafka_consumer(mut task_manager:TaskManager) {    
    let mut consumer = ClientConfig::new()
        .set("group.id", "test-group")
        .set("bootstrap.servers", "")
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        //.set("statistics.interval.ms", "30000")
        //.set("auto.offset.reset", "smallest")
        .set_log_level(RDKafkaLogLevel::Debug)

    consumer
        .subscribe("spread")
        .expect("Can't subscribe to specified topics");

    loop {
        match consumer.recv().await {
            Err(e) => println!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        println!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };
                println!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
                if let Some(headers) = m.headers() {
                    for header in headers.iter() {
                        println!("  Header {:#?}: {:?}", header.key, header.value);
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}
 */

#[get("/api/test")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str = "Build Simple CRUD API with Rust and Actix Web";

    let task = TASK_MAP.lock().unwrap();
    if let Some(task_handler) = task.get("061a8bd0-a84b-11ed-abb7-27c66719279e") {
        task_handler.abort();
    }

    let response_json = &GenericResponse {
        status: "success".to_string(),
        message: MESSAGE.to_string(),
    };


    HttpResponse::Ok().json(response_json)
}


#[get("/api/test2")]
async fn health_checker_handler2() -> impl Responder {
    const MESSAGE: &str = "test2";
    let mongo_uri = env::var("MONGO_URI").unwrap();
    let mongo_db_name = env::var("MONGO_DB_NAME").unwrap();    
    let fclt_obj = fclt::FcltLib::new(mongo_uri, mongo_db_name).await;
    let fclt_data = fclt_obj.get_fclt_one("061a8bd0-a84b-11ed-abb7-27c66719279e".to_string()).await;    

    thread_proc(fclt_data);

    let response_json = &GenericResponse {
        status: "success".to_string(),
        message: MESSAGE.to_string(),
    };


    HttpResponse::Ok().json(response_json)
}


// #[tokio::main]
#[actix_web::main]
async fn main()  -> std::io::Result<()> { 
    let path = util::get_env_path().expect("Couldn't");    
    let server_ip = String::from(env::var("SERVER_ADDR").unwrap_or("0.0.0.0".to_string()));    
    let port : u16 = env::var("SERVER_PORT").unwrap_or("8000".to_string()).parse().unwrap();

    println!("{}", path.display());
    dotenv::from_filename(path).expect("Failed to open directory");    

    println!("Boot on Server Mode");
    server_mode().await.unwrap();            
    HttpServer::new(move || {        
        App::new()
            .service(health_checker_handler)
            .service(health_checker_handler2)
            .wrap(Logger::default())
    })
    .bind((server_ip, port))?
    .run()
    .await
}

