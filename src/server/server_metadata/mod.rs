extern crate chrono;
extern crate image;

use std::sync::{Arc};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;

use tokio::sync::Mutex;
use std::collections::HashMap;
use futures::StreamExt;
use url::{Url};
use anyhow::{anyhow, Error};

use retina::codec::CodecItem;
use retina::client::{SessionGroup, SetupOptions};
use async_trait::async_trait;
use uuid::Uuid;
use lazy_static::lazy_static; 
use serde_json::{Value,json};

use reqwest::Client; 
use quickxml_to_serde::{xml_string_to_json, Config,NullValue};
use chrono::*;

pub mod request;
pub mod lpr;

use crate::server_metadata;

#[async_trait]
pub trait MetadataManager {
  async fn run_onvif(&self) -> Result<(), Error> ;   
  async fn hanwha_proc(&self, json:Value) -> Result<(), Error> ;
  async fn truen_proc(&self, json:Value) -> Result<(), Error>;
  async fn save_bestshot(img_save_path:String, ip:String, image_ref:String) -> Result<(), Error>;
  async fn save_truen_bestshot(img_save_path:String, ip:String, image_ref:String) -> Result<(), Error>;
}

pub struct Metadata {  
  pub camera_ip: String,  
  pub rtsp_port: String,  
  pub username: String,  
  pub password: String,
  pub img_save_path: String,
  pub fclt_id: String,
  pub ai_cam_model: String,
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

lazy_static! {
    static ref METADATA_MAP: Arc<Mutex<HashMap<u64, MetadataObject<>>>> = Arc::new(Mutex::new(HashMap::new()));
}

#[async_trait]
impl MetadataManager for Metadata {  
  async fn run_onvif(&self) -> Result<(), Error> {    
    // hanwha
    // let rtsp_url = format!("rtsp://{}/profile1/media.smp", self.url.clone());

    // truen

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
                            Self::hanwha_proc(self, json).await.unwrap();
                        }
                        else if self.ai_cam_model.contains("truen") {
                            Self::truen_proc(self, json).await.unwrap();
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
  async fn hanwha_proc(&self, json:Value) -> Result<(), Error> {

    let meta = json["MetadataStream"]["VideoAnalytics"].clone();
    let date_str:String = meta["Frame"]["UtcTime"].to_string().replace("\"", "");
               

    if date_str != "null" {
        let date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
        // 객체가 한 개일때
        if meta["Frame"]["Object"].as_array().iter().len() == 0 {            
            let cloned_data = meta["Frame"]["Object"].clone();
            if !cloned_data.is_null() {                                
                let object_id = cloned_data["ObjectId"].clone();
                let mut metadata_object = MetadataObject{
                    object_id: object_id.to_string(),
                    class: cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string().replace("\"", ""),
                    object_array: json!(meta["Frame"]["Object"].clone()),
                    last_time: date.timestamp_millis(),
                    cross_line: vec![]
                };                                                                       
                let clone1 = Arc::clone(&METADATA_MAP);
                if !clone1.lock().await.contains_key(&object_id.as_u64().unwrap()) {
                    clone1.lock().await.insert(object_id.as_u64().unwrap(), metadata_object);
                }                                    
                else {
                    clone1.lock().await.entry(object_id.as_u64().unwrap()).and_modify(|e| { *e = metadata_object});
                }
                
                let metadata_class = cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string();
                if let Some(image_ref) = cloned_data["Appearance"].get("ImageRef") {
                    
                    Self::save_bestshot(self.img_save_path.clone(), self.camera_ip.clone().to_string(), image_ref.to_string().replace("\"", "")).await.unwrap();                    

                }
                // 얼굴일 때 안면 분석 쓰레드 돌림
                if metadata_class.contains("Head") {
                    if !cloned_data["Appearance"]["ImageRef"].is_null() {
                        let file_name = Uuid::new_v4();
                        request::fetch_url("a".to_string(), file_name.to_string()).await.unwrap();
                    }
                }
                else if metadata_class.contains("Human") {
                
                }
       
                else if metadata_class.contains("Vehicle") {
                    
                }
                // 자동차 번호판일 때 차량번호 판독 모듈 실행
                else if metadata_class.contains("LicensePlate") {
                
                }
            }                
           
        }
        // 객체가 여러개일 때
        else {
            for objects in meta["Frame"]["Object"].as_array().into_iter() {
                for obj in objects.iter() {
                    let cloned_data = obj.clone();
                    let object_id = cloned_data["ObjectId"].clone();
                    let mut metadata_object = MetadataObject{
                        object_id: object_id.to_string(),
                        class: cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string().replace("\"", ""),
                        object_array: json!(obj.clone()),
                        last_time: date.timestamp_millis(),
                        cross_line: vec![]
                    };                                                                       
                    let clone1 = Arc::clone(&METADATA_MAP);
                    if !clone1.lock().await.contains_key(&object_id.as_u64().unwrap()) {
                        clone1.lock().await.insert(object_id.as_u64().unwrap(), metadata_object);
                    }                                    
                    else {
                        clone1.lock().await.entry(object_id.as_u64().unwrap()).and_modify(|e| { *e = metadata_object});
                    }
                    
                    let metadata_class = cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string();
                    if let Some(image_ref) = cloned_data["Appearance"].get("ImageRef") {
                    
                        Self::save_bestshot(self.img_save_path.clone(), self.camera_ip.clone().to_string(), image_ref.to_string().replace("\"", "")).await.unwrap();                        
    
                    }
                    // 얼굴일 때 안면 분석 쓰레드 돌림
                    if metadata_class.contains("Head") {
                        if !cloned_data["Appearance"]["ImageRef"].is_null() {
                            let file_name = Uuid::new_v4();
                            request::fetch_url("a".to_string(), file_name.to_string()).await.unwrap();
                        }
                    }
                    else if metadata_class.contains("Human") {
                       
                    }
     
                    else if metadata_class.contains("Vehicle") {
                        
                    }
                    // 자동차 번호판일 때 차량번호 판독 모듈 실행
                    else if metadata_class.contains("LicensePlate") {
                     
                    }
    
                    
                }                                 
            }
        }                
    }   
    Ok(())
  }

//   async fn truen_proc(&self, json:Value) -> Result<(), Error> {
//     Ok(())
//   }

  async fn truen_proc(&self, json:Value) -> Result<(), Error> {

    let meta1 = json["MetaDataStream"]["Event"]["NotificationMessage"].clone(); 
    let meta = json["MetaDataStream"]["VideoAnalytics"].clone();
    let date_str:String = meta["Frame"]["UtcTime"].to_string().replace("\"", "");
    
    if date_str != "null" {
        let date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
        // 객체가 한 개일때
        if meta["Frame"]["Object"].as_array().iter().len() == 0 {            
            let cloned_data = meta["Frame"]["Object"].clone();

            if !cloned_data.is_null() {                                
                let object_id = cloned_data["ObjectId"].clone();
                let mut metadata_object = MetadataObject{
                    object_id: object_id.to_string(),
                    class: cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string().replace("\"", ""),
                    object_array: json!(meta["Frame"]["Object"].clone()),
                    last_time: date.timestamp_millis(),
                    cross_line: vec![]
                };   
                                                                    
                let clone1 = Arc::clone(&METADATA_MAP);
                if !clone1.lock().await.contains_key(&object_id.as_u64().unwrap()) {
                    clone1.lock().await.insert(object_id.as_u64().unwrap(), metadata_object);
                }                                    
                else {
                    clone1.lock().await.entry(object_id.as_u64().unwrap()).and_modify(|e| { *e = metadata_object});
                }
                
                let metadata_class = cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string();
                if let Some(image_ref) = cloned_data["Appearance"].get("ImageRef") {
 
                    Self::save_truen_bestshot(self.img_save_path.clone(), self.camera_ip.clone().to_string(), image_ref.to_string().replace("\"", "")).await.unwrap();                    

                }
                // 얼굴일 때 안면 분석 쓰레드 돌림
                if metadata_class.contains("Head") {
                    if !cloned_data["Appearance"]["ImageRef"].is_null() {
                        let file_name = Uuid::new_v4();
                        // request::fetch_url("a".to_string(), file_name.to_string()).await.unwrap();
                    }
                }
                else if metadata_class.contains("Human") {
                
                }
       
                else if metadata_class.contains("Vehicle") {
                    
                }
                // 자동차 번호판일 때 차량번호 판독 모듈 실행
                else if metadata_class.contains("LicensePlate") {
                
                }
            }                
           
        }
        // 객체가 여러개일 때
        else {
            for objects in meta["Frame"]["Object"].as_array().into_iter() {
                for obj in objects.iter() {
                    let cloned_data = obj.clone();
                    let object_id = cloned_data["ObjectId"].clone();
                    let mut metadata_object = MetadataObject{
                        object_id: object_id.to_string(),
                        class: cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string().replace("\"", ""),
                        object_array: json!(obj.clone()),
                        last_time: date.timestamp_millis(),
                        cross_line: vec![]
                    };                                                                       
                    let clone1 = Arc::clone(&METADATA_MAP);
                    if !clone1.lock().await.contains_key(&object_id.as_u64().unwrap()) {
                        clone1.lock().await.insert(object_id.as_u64().unwrap(), metadata_object);
                    }                                    
                    else {
                        clone1.lock().await.entry(object_id.as_u64().unwrap()).and_modify(|e| { *e = metadata_object});
                    }
                    
                    let metadata_class = cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string();
                    if let Some(image_ref) = cloned_data["Appearance"].get("ImageRef") {
                        println!("{:?}", image_ref); 
                        Self::save_truen_bestshot(self.img_save_path.clone(), self.camera_ip.clone().to_string(), image_ref.to_string().replace("\"", "")).await.unwrap();                        
    
                    }
                    // 얼굴일 때 안면 분석 쓰레드 돌림
                    if metadata_class.contains("Head") {
                        if !cloned_data["Appearance"]["ImageRef"].is_null() {
                            let file_name = Uuid::new_v4();
                            // request::fetch_url("a".to_string(), file_name.to_string()).await.unwrap();
                        }
                    }
                    else if metadata_class.contains("Human") {
                       
                    }
     
                    else if metadata_class.contains("Vehicle") {
                        
                    }
                    // 자동차 번호판일 때 차량번호 판독 모듈 실행
                    else if metadata_class.contains("LicensePlate") {
                     
                    }
    
                    
                }                                 
            }
        }   
    }
    Ok(())
  }




  async fn save_truen_bestshot(img_save_path:String, ip:String, image_ref:String) -> Result<(), Error> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let file_path = format!("{}/{}",img_save_path, ip);
    let file_name = format!("{}/{:?}.jpg", file_path.clone(), time);
    let return_file_name = file_name.to_owned();
    // tokio::spawn(async move {        

    fs::create_dir_all(file_path.clone()).unwrap();        
    let url = format!("{}", image_ref);
    let client = Client::new();                                            

    let resp = client.get(url).send().await.unwrap();

    match resp.error_for_status() {
        Ok(_res) => {
            let img_bytes = _res.bytes().await.unwrap();                                                                                        
            let img = image::load_from_memory(&img_bytes).unwrap();        
            img.save(file_name).unwrap();                                
        },
        Err(err) => {
            // asserting a 400 as an example
            // it could be any status between 400...599
            assert_eq!(
                err.status(),
                Some(reqwest::StatusCode::BAD_REQUEST)
            );
        }
    }
    
    Ok(())
  }


  async fn save_bestshot(img_save_path:String, ip:String, image_ref:String) -> Result<(), Error> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let file_path = format!("{}/{}",img_save_path, ip);
    let file_name = format!("{}/{:?}.jpg", file_path.clone(), time);
    let return_file_name = file_name.to_owned();
    // tokio::spawn(async move {        

    fs::create_dir_all(file_path.clone()).unwrap();        
    let url = format!("http://{}{}", ip, image_ref);
    let client = Client::new();                                            

    let resp = client.get(url).send().await.unwrap();

    match resp.error_for_status() {
        Ok(_res) => {
            let img_bytes = _res.bytes().await.unwrap();                                                                                        
            let img = image::load_from_memory(&img_bytes).unwrap();        
            img.save(file_name).unwrap();                                
        },
        Err(err) => {
            // asserting a 400 as an example
            // it could be any status between 400...599
            assert_eq!(
                err.status(),
                Some(reqwest::StatusCode::BAD_REQUEST)
            );
        }
    }
    
    Ok(())
  }    
}

