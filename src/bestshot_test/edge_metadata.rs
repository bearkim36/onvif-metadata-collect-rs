extern crate chrono;

use std::{env, fs};

use std::sync::Arc;
use futures::StreamExt;
use reqwest::Client;
use url::Url;
use anyhow::{anyhow, Error};

use retina::codec::CodecItem;
use retina::client::{SessionGroup, SetupOptions};
use async_trait::async_trait;
use tokio::task;

use quickxml_to_serde::{xml_string_to_json, Config,NullValue};
use uuid::Uuid;


#[async_trait]
pub trait MetadataManager {
  async fn run_onvif(&self, analysis_url:String,  image_path:String) -> Result<(), Error> ;   
}

pub struct Metadata {  
  pub url: String,  
  pub username: String,  
  pub password: String,
}

// use crate::edge_metadata;

#[async_trait]
impl MetadataManager for Metadata {  
  async fn run_onvif(&self, analysis_url:String, image_path:String) -> Result<(), Error> {

    let session_group = Arc::new(SessionGroup::default());
    let creds = Some(retina::client::Credentials {
        username : self.username.clone(),
        password: self.password.clone(),
    });
    let mut session = retina::client::Session::describe(
        Url::parse(self.url.as_str())?,
        retina::client::SessionOptions::default()
            .creds(creds)
            .user_agent("Bestshot test".to_owned())
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
                        let cloned_image_path = image_path.clone();                 
                        let conf = Config::new_with_custom_values(true, "", "txt", NullValue::Null);
                        let json = xml_string_to_json(std::str::from_utf8(m.data()).unwrap().to_string(), &conf).unwrap();
                        let image_ref = json["MetaDataStream"]["VideoAnalytics"]["Frame"]["Object"]["Appearance"]["ImageRef"].to_string();
                        
                        if image_ref != "null" {
                            task::spawn(async move {                                
                                let file_name = format!("{}/{:?}.jpg", cloned_image_path.clone(), Uuid::new_v4());
                                let return_file_name = file_name.to_owned();
                            
                                fs::create_dir_all(cloned_image_path.clone()).unwrap();        
                                let url = format!("{}", image_ref);
                                let client = Client::new();                                            
                                println!("{:?}", url.replace("\"", "").replace("172.30.1.11", "southdoor2.truecam.net"));                                
                                let resp = client.get(url.replace("\"", "").replace("172.30.1.11", "southdoor2.truecam.net")).send().await.unwrap();
                            
                                match resp.error_for_status() {
                                    Ok(_res) => {
                                        let img_bytes = _res.bytes().await.unwrap();                                                                                        
                                        fs::write(file_name, img_bytes).expect("Unable to write file");                                        
                                    },
                                    Err(err) => {
                                        assert_eq!(
                                            err.status(),
                                            Some(reqwest::StatusCode::BAD_REQUEST)
                                        );
                                    }
                                }
                            });
                        }


                        // let _sub_thread = task::spawn( async  {   
                        //     let client = reqwest::Client::new();
                        //     let _res = client.post(analysis_url)
                        //         .header("content-type", "application/json")
                        //         .body(param)
                        //         .send()
                        //         .await.unwrap_or_else(|error| {
                        //             panic!("occured network: {:?}", error);
                        //         });

                        // });
                    },
                    _ => continue,
                };
            },           
        }   
    }
 
  }

  

}
