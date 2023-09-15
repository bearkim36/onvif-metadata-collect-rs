extern crate chrono;

use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Error;
use chrono::NaiveDateTime;
use serde_json::Value;
use uuid::Uuid;
use std::fs;
use rdkafka::producer::{FutureProducer, FutureRecord};
use reqwest::Client; 
use chrono::Local;

use crate::server_metadata::metadata;
use crate::server_metadata::bestshot;

pub async fn proc(json:Value, producer:FutureProducer, fclt_id:String, camera_ip:String, http_port:String, img_save_path:String) -> Result<metadata::Metadata, Error> {
    let meta = json["MetaDataStream"]["VideoAnalytics"].clone();
    let date_str:String = meta["Frame"]["UtcTime"].to_string().replace("\"", "");
    let metadata_result:metadata::Metadata = metadata::Metadata::new();

    if date_str != "null" {
        let date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
        // 객체가 한 개일때
        if meta["Frame"]["Object"].as_array().iter().len() == 0 {            
            let cloned_data = meta["Frame"]["Object"].clone();

            if !cloned_data.is_null() {                                
                proc_metadata(cloned_data, producer, fclt_id, date_str,  camera_ip, http_port, img_save_path).await.unwrap();
            }                
           
        }
        // 객체가 여러개일 때
        else {
            for objects in meta["Frame"]["Object"].as_array().into_iter() {
                for obj in objects.iter() {
                    let cloned_data = obj.clone();
                    proc_metadata(cloned_data, producer.clone(), fclt_id.clone(), date_str.clone(), camera_ip.clone(), http_port.clone(), img_save_path.clone()).await.unwrap();         
                }
            }
        }   
    }
    Ok(metadata_result)
  }

  async fn proc_metadata(metadata:Value, producer:FutureProducer, utc_time:String, fclt_id:String, camera_ip:String, http_port:String, img_save_path:String) -> Result<(), Error> {
    let cloned_data = metadata.clone();
    let object_id = cloned_data["ObjectId"].clone();
    let mut metadata_result:metadata::Metadata = metadata::Metadata::new();
                                                        

    let metadata_class = cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string();
    if let Some(image_ref) = cloned_data["Appearance"].get("ImageRef") {
        let date_format = Local::now().format("%Y-%m-%d").to_string();
        let file_name = format!("{}.jpg" ,Uuid::new_v4());
        let file_path = format!("{}/{}/{}/{}", img_save_path, date_format, fclt_id, file_name);
        let bestshot = bestshot::Bestshot {
          fclt_id,
          image_ref: image_ref.to_string().replace("\"", ""),
          object_id: object_id.to_string(),
          camera_ip: camera_ip.to_string(),
          http_port: http_port.to_string(),
          utc_time,
          class: metadata_class.to_string().replace("\"", ""),
          date: date_format,
          file_name: file_name
        };    
        metadata_result.imagePath = file_path;
          let bestshot_buffer = serde_json::to_string(&bestshot).expect("json serialazation failed");
          producer.send(
            FutureRecord::to("bestshot")
                .key(&object_id.to_string())
                .payload(&bestshot_buffer),
                std::time::Duration::from_secs(0)
          ).await.unwrap();       
    }
    
    
    Ok(())
  }


  async fn save_bestshot(img_save_path:String, ip:String, image_ref:String) -> Result<String, Error> {    
    let file_path = format!("{}/{}",img_save_path, ip);
    let file_name = format!("{}/{:?}.jpg", file_path.clone(), Uuid::new_v4());
    let return_file_name = file_name.to_owned();

    fs::create_dir_all(file_path.clone()).unwrap();        
    let url = format!("{}", image_ref);
    let client = Client::new();                                            

    let resp = client.get(url).send().await.unwrap();

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
    
    Ok(return_file_name)
  }

