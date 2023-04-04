extern crate chrono;
extern crate image;

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
use serde_json::{Value,json};

// use reqwest::Client; 
use quickxml_to_serde::{xml_string_to_json, Config,NullValue};
// use chrono::*;

mod request;
mod lpr;
mod hanwha;
mod truen;
mod facerecognition;

// use crate::server_metadata;

#[async_trait]
pub trait MetadataManager {
  async fn run_onvif(&self) -> Result<(), Error> ;   
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
#[derive(Debug)]
pub struct MetadataObject {    
    pub object_id: String,
    pub class: String,
    pub object_array: Value,
    pub last_time: i64,
    pub cross_line: Vec<String>,
}

// lazy_static! {
//     static ref METADATA_MAP: Arc<Mutex<HashMap<u64, MetadataObject<>>>> = Arc::new(Mutex::new(HashMap::new()));
// }

#[async_trait]
impl MetadataManager for MetadataConfig {  
  async fn run_onvif(&self) -> Result<(), Error> {    
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

    
    loop {
        tokio::select! {
            item = session.next() => {
                match item.ok_or_else(|| anyhow!("EOF"))?? {
                    CodecItem::MessageFrame(m) => {
                        //println!("{}", std::str::from_utf8(m.data()).unwrap());
                        let conf = Config::new_with_custom_values(true, "", "txt", NullValue::Null);
                        let json = xml_string_to_json(std::str::from_utf8(m.data()).unwrap().to_string(), &conf).unwrap();
        
                        if self.ai_cam_model.contains("hanwha") {
                            hanwha::proc(json, self.camera_ip.clone(), self.http_port.clone(), self.img_save_path.clone(), self.face_recognition_url.clone()).await.unwrap();
                        }
                        else if self.ai_cam_model.contains("truen") {
                            truen::proc(json, self.camera_ip.clone(), self.http_port.clone(), self.img_save_path.clone(), self.face_recognition_url.clone()).await.unwrap();
                        }

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

