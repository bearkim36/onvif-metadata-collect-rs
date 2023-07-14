extern crate chrono;

use std::sync::{Arc};
// use std::time::{SystemTime, UNIX_EPOCH};

// use tokio::sync::Mutex;
// use std::collections::HashMap;
use futures::StreamExt;
use url::{Url};
use anyhow::{anyhow, Error};

use retina::codec::CodecItem;
use retina::client::{SessionGroup, SetupOptions};
use async_trait::async_trait;
// use uuid::Uuid;
// use lazy_static::lazy_static; 

use serde::{Deserialize, Serialize};

// use reqwest::Client; 
use quickxml_to_serde::{xml_string_to_json, Config,NullValue};
// use chrono::*;

use rdkafka::producer::{FutureProducer, FutureRecord};



mod request;
mod lpr;
mod hanwha;
mod truen;
mod facerecognition;
mod metadata;
// use crate::server_metadata;

#[async_trait]
pub trait MetadataManager {
  async fn run_onvif(&self, producer:FutureProducer) -> Result<(), Error> ;   
//   async fn hanwha_proc(&self, json:Value) -> Result<(), Error> ;
//   async fn truen_proc(&self, json:Value) -> Result<(), Error>;
//   async fn save_bestshot(img_save_path:String, ip:String, image_ref:String) -> Result<(), Error>;
//   async fn save_truen_bestshot(img_save_path:String, ip:String, image_ref:String) -> Result<(), Error>;
}

pub struct MetadataConfig {  
  pub camera_ip: String,  
  pub http_port: String,  
  pub rtsp_port: String,  
  pub username: String,  
  pub password: String,
  pub img_save_path: String,
  pub fclt_id: String,
  pub ai_cam_model: String,
  pub face_recognition_url: String  
}

#[derive(Clone)]
#[derive(Serialize, Deserialize ,Debug)]
pub struct MetadataObject {    
    pub object_id: String,    
}

// lazy_static! {
//     static ref METADATA_MAP: Arc<Mutex<HashMap<u64, MetadataObject<>>>> = Arc::new(Mutex::new(HashMap::new()));
// }

#[async_trait]
impl MetadataManager for MetadataConfig {  
  async fn run_onvif(&self, producer:FutureProducer) -> Result<(), Error> {        
  
    let mut rtsp_url = "".to_string();
    if self.ai_cam_model.contains("hanwha") {
        rtsp_url = std::format!("rtsp://{}:{}/profile1/media.smp", self.camera_ip, self.rtsp_port);
    }
    else if self.ai_cam_model.contains("truen") {
        rtsp_url = std::format!("rtsp://{}:{}/video1", self.camera_ip, self.rtsp_port);
    }    
    println!("onvif url: {}", rtsp_url.clone());

    let session_group = Arc::new(SessionGroup::default());
    let creds = Some(retina::client::Credentials {
        username : self.username.clone(),
        password: self.password.clone(),
    });
    let mut session = retina::client::Session::describe(
        Url::parse(&rtsp_url.to_string())?,
        retina::client::SessionOptions::default()
            .creds(creds)
            .user_agent("DK Edge metadata".to_owned())
            .session_group(session_group),
    )
    .await?;
    let onvif_stream_i = session
        .streams()
        .iter()
        .position(|s| {
            matches!(
                s.parameters(),
                Some(retina::codec::ParametersRef::Message(..))
            )
        })
        .ok_or_else(|| anyhow!("couldn't find onvif stream"))?;
    session
        .setup(onvif_stream_i, SetupOptions::default())
        .await?;
    let mut session = session
        .play(retina::client::PlayOptions::default().ignore_zero_seq(true))
        .await?
        .demuxed()?;

    let mut i = 0;
    
    loop {
        i += 1;
        tokio::select! {
            item = session.next() => {
                match item.ok_or_else(|| anyhow!("EOF"))?? {
                    CodecItem::MessageFrame(m) => {
                        //println!("{}", std::str::from_utf8(m.data()).unwrap());
                        let conf = Config::new_with_custom_values(true, "", "txt", NullValue::Null);
                        let json = xml_string_to_json(std::str::from_utf8(m.data()).unwrap().to_string(), &conf).unwrap();
                        let mut metadata_object = metadata::Metadata {
                            faceId: "".to_string(),
                            fcltId: "".to_string(),
                            rect: metadata::Rect {
                                top: 0,
                                bottom: 0,
                                left: 0,
                                right: 0,
                                center: metadata::XY {
                                    x: 0,
                                    y: 0,
                                },
                                translate: metadata::XY {
                                  x: 0,
                                  y: 0,
                              },
                            },
                            class: metadata::MetadataClass {
                                r#type: "".to_string(),
                                likelihood: "".to_string(),
                            },
                            currentTime: 0,
                            plateNumberDetecting: false,
                            plateUuid: "".to_string(),
                            detectStatus: "".to_string(),
                            detectType: 0,
                            vehicleType: 0,
                        }; 

                        if self.ai_cam_model.contains("hanwha") {
                            metadata_object = hanwha::proc(json, self.camera_ip.clone(), self.http_port.clone(), self.img_save_path.clone(), self.face_recognition_url.clone()).await.unwrap();
                        }
                        else if self.ai_cam_model.contains("truen") {
                            metadata_object = truen::proc(json, self.camera_ip.clone(), self.http_port.clone(), self.img_save_path.clone(), self.face_recognition_url.clone()).await.unwrap();
                        }
                        metadata_object.fcltId = i.to_string();
                        

                        let metadata_object_buffer = serde_json::to_string(&metadata_object).expect("json serialazation failed");

                        producer.send(
                            FutureRecord::to("metadata")
                                .key(&i.to_string())
                                .payload(&metadata_object_buffer),
                                std::time::Duration::from_secs(0)
                        ).await.unwrap();
                    },
                    _ => continue,
                };
            },
           
        }   
    }     
  }


//   fn clear_data() {    
//     tokio::spawn(async move {                
//         let clone2 = Arc::clone(&METADATA_MAP);
//       //  println!("{}", clone2.lock().await.len());        
//         let mut keys:Vec<u64> = Vec::new();
//         let current_time = Utc::now().timestamp_millis();
//         for (k, v) in METADATA_MAP.lock().await.iter() {
//             if (current_time - v.last_time) > 15000 {
//                 keys.push(*k);
//             }
//         }        
//         for k in keys {
//             clone2.lock().await.remove(&k);
//         }                                      
//     });            
//   }
  
}

