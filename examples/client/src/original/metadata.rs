extern crate chrono;

use std::hash::Hash;
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use futures::StreamExt;
use url::{Url};
use anyhow::{anyhow, Error};

use retina::codec::CodecItem;
use retina::client::{SessionGroup, SetupOptions};
use async_trait::async_trait;
use uuid::Uuid;
use lazy_static::lazy_static; 
use serde_json::{Value,json,Number};

use quickxml_to_serde::{xml_string_to_json, Config,NullValue};
use chrono::*;

use crate::metadata;

#[async_trait]
pub trait MetadataManager {
  async fn run_onvif(&self) -> Result<(), Error> ; 
  fn clear_data(m: &HashMap<u64, MetadataObject<>>);
}

pub struct Metadata {  
  pub url: String,  
  pub username: String,  
  pub password: String,
  pub fclt_name: String,
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
    static ref METADATA_MAP:HashMap<u64, MetadataObject<>> =  HashMap::new();
}

#[async_trait]
impl MetadataManager for Metadata {  
  async fn run_onvif(&self) -> Result<(), Error> {
    let mut metadata_map: HashMap<u64, MetadataObject<>> = HashMap::new();
    let mut clone1 = metadata_map.clone();
    let mut clone2 = metadata_map.clone();

    println!("onvif thread activation [{}]", self.fclt_name.clone());
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
            .user_agent("Retina metadata example".to_owned())
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

     //   thread::spawn(move || metadata::Metadata::clear_data(&clone1));
           
 
    loop {
        tokio::select! {
            item = session.next() => {
                match item.ok_or_else(|| anyhow!("EOF"))?? {
                    CodecItem::MessageFrame(m) => {
                        let current_time = Utc::now().timestamp_millis();
                       


                        //println!("{}", std::str::from_utf8(m.data()).unwrap());
                        let conf = Config::new_with_custom_values(true, "", "txt", NullValue::Null);
                        let json = xml_string_to_json(std::str::from_utf8(m.data()).unwrap().to_string(), &conf).unwrap();
                        let meta = json["MetadataStream"]["VideoAnalytics"].clone();
                        let date_str:String = meta["Frame"]["UtcTime"].to_string().replace("\"", "");                                                                                                         
                        for (key, value) in metadata_map.into_iter() {
                            if current_time - value.last_time > 5000 {
                                println!("{}", clone1.len());
                                //  m.lock().unwrap().clear();
                            }
                        }                          

                        if date_str != "null" {
                            let date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();                            
                            for objects in meta["Frame"]["Object"].as_array().into_iter() {
                                for obj in objects.iter() {
                                    let cloned_data = obj.clone();                                    
                                    let object_id = cloned_data["ObjectId"].clone();
                                    let mut metadata_object = MetadataObject{
                                        object_id: object_id.to_string(),
                                        class: cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string(),
                                        object_array: json!(obj.clone()),
                                        last_time: date.timestamp_millis(),
                                        cross_line: vec![]
                                    };
                                    if !clone1.contains_key(&object_id.as_u64().unwrap()) {
                                        clone1.insert(object_id.as_u64().unwrap(), metadata_object);
                                    }
                                    else {
                                        clone1.entry(object_id.as_u64().unwrap()).and_modify(|e| { *e = metadata_object});
                                    }                                                                                                   
                                   println!("metadata: {}", clone1.get(&object_id.as_u64().unwrap()).unwrap().last_time);
                                    

                                }                                 
                            }                              
                        }                
                        
        
                        
                        // println!("{}", json.unwrap());

                        
                    //    println!"%Y-%m-%dT%H:%M:%S"                    //         "{}: {}\n",
                    //         &m.timestamp(),
                    //         std::str::from_utf8(m.data()).unwrap(),
                    //     );                       
                 
                    },
                    _ => continue,
                };
            },
           
        }
      
    }
    
    
  }
  fn clear_data(m: &HashMap<u64, MetadataObject<>>) {
    loop {
        let current_time = Utc::now().timestamp_millis();
        for (key, value) in m {
            if current_time - value.last_time > 5000 {
                println!("{}", m.len());
                //  m.lock().unwrap().clear();
            }
        }
    }
  }
}
