extern crate chrono;
extern crate image;

use std::sync::{Arc};
use std::time::{SystemTime, UNIX_EPOCH};

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
  fn clear_data();
  fn save_bestshot(ip:String, image_ref:String);
}

pub struct Metadata {  
  pub url: String,  
  pub username: String,  
  pub password: String,
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

    println!("onvif url: {}", self.url.clone());
    println!("onvif id/pw {}/{}", self.username.clone(), self.password.clone());    
    let session_group = Arc::new(SessionGroup::default());
    let creds = Some(retina::client::Credentials {
        username : self.username.clone(),
        password: self.password.clone(),
    });
    let mut session = retina::client::Session::describe(
        Url::parse(self.url.as_str())?,
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

    //
    loop {
        tokio::select! {
            item = session.next() => {
                match item.ok_or_else(|| anyhow!("EOF"))?? {
                    CodecItem::MessageFrame(m) => {
                        //println!("{}", std::str::from_utf8(m.data()).unwrap());
                        let conf = Config::new_with_custom_values(true, "", "txt", NullValue::Null);
                        let json = xml_string_to_json(std::str::from_utf8(m.data()).unwrap().to_string(), &conf).unwrap();
                        let meta = json["MetadataStream"]["VideoAnalytics"].clone();
                        let date_str:String = meta["Frame"]["UtcTime"].to_string().replace("\"", "");
                                   

                        if date_str != "null" {
                            let date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();                            
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
                                        Self::save_bestshot("192.168.0.16".to_string(), image_ref.to_string().replace("\"", ""));

                                    }
                                    // 얼굴일 때 안면 분석 쓰레드 돌림
                                    if metadata_class.eq("face") {
                                        tokio::spawn(async move {
                                            let file_name = Uuid::new_v4();
                                            request::fetch_url("a".to_string(), file_name.to_string()).await.unwrap();

                                        });
                                    }
                                    else if metadata_class.eq("Human") {
                                       
                                    }
                                    else if metadata_class.eq("Head") {
                                        
                                    }
                                    else if metadata_class.eq("Vehicle") {
                                        
                                    }
                                    // 자동차 번호판일 때 차량번호 판독 모듈 실행
                                    else if metadata_class.eq("LicensePlate") {
                                     
                                    }

                                    
                                }                                 
                            }                              
                        }             
                        

                        server_metadata::Metadata::clear_data();                                 
                    },
                    _ => continue,
                };
            },
           
        }   
    }
 
  }


  fn clear_data() {    
    tokio::spawn(async move {                
        let clone2 = Arc::clone(&METADATA_MAP);
      //  println!("{}", clone2.lock().await.len());        
        let mut keys:Vec<u64> = Vec::new();
        let current_time = Utc::now().timestamp_millis();
        for (k, v) in METADATA_MAP.lock().await.iter() {
            if (current_time - v.last_time) > 15000 {
                keys.push(*k);
            }
        }        
        for k in keys {
            clone2.lock().await.remove(&k);
        }                                      
    });            
  }

  fn save_bestshot(ip:String, image_ref:String) {
    tokio::spawn(async move {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let file_name = format!("/home/bearkim/collect_test/{:?}.jpg", time);
        let url = format!("http://{}/{}", ip, image_ref);
        let client = Client::new();                                            

        let resp = client.get(url).send().await.unwrap();
        let img_bytes = resp.bytes().await.unwrap();                                                                                        
        let img = image::load_from_memory(&img_bytes).unwrap();

        img.save(file_name).unwrap();
    });
  }
}




