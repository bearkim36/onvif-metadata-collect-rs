use anyhow::Error;
use chrono::NaiveDateTime;
use serde_json::Value;
use uuid::Uuid;
use std::fs;
use reqwest::Client; 

use crate::server_metadata::facerecognition;
use crate::server_metadata::metadata;

pub async fn proc(json:Value,  camera_ip:String, http_port:String, img_save_path:String, face_recognition_url:String) -> Result<metadata::Metadata, Error> {
  let meta = json["MetadataStream"]["VideoAnalytics"].clone();
  let date_str:String = meta["Frame"]["UtcTime"].to_string().replace("\"", "");
  let metadata_result:metadata::Metadata = metadata::Metadata {
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

  if date_str != "null" {
      let date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
      // 객체가 한 개일때      
      if meta["Frame"]["Object"].as_array().iter().len() == 0 {            
          let cloned_data = meta["Frame"]["Object"].clone();
          if !cloned_data.is_null() {                                
            proc_metadata(cloned_data, camera_ip, http_port, img_save_path, face_recognition_url).await.unwrap();
          }                
         
      }
      // 객체가 여러개일 때
      else {
          for objects in meta["Frame"]["Object"].as_array().into_iter() {
              for obj in objects.iter() {
                  let cloned_data = obj.clone();
                  proc_metadata(cloned_data,  camera_ip.clone(), http_port.clone(), img_save_path.clone(), face_recognition_url.clone()).await.unwrap();
              }
          }
      }                
  }   
  Ok(metadata_result)
}

async fn proc_metadata(metadata:Value, camera_ip:String, http_port:String, img_save_path:String, face_recognition_url:String) -> Result<(), Error> {  
  let cloned_data = metadata.clone();  
  let object_id = cloned_data["ObjectId"].clone();

  // let mut metadata_object = MetadataObject{
  //     object_id: object_id.to_string(),
  //     class: cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string().replace("\"", ""),
  //     object_array: json!(meta["Frame"]["Object"].clone()),
  //     last_time: date.timestamp_millis(),
  //     cross_line: vec![]
  // };
  let metadata_class = cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string();

  // 메타데이터 처리
  if metadata_class.contains("Head") {    

  }
  else if metadata_class.contains("Human") {
    
  }

  else if metadata_class.contains("Vehicle") {
      
  }  
  else if metadata_class.contains("LicensePlate") {

  }
  
  let mut save_file_name:String = "".to_string();
  if let Some(image_ref) = cloned_data["Appearance"].get("ImageRef") {        
    save_file_name = save_bestshot(img_save_path, camera_ip, http_port, image_ref.to_string().replace("\"", "")).await.unwrap();
    if metadata_class.contains("Face") {
        let face_result = facerecognition::recog(save_file_name, face_recognition_url).await.unwrap();
                        
        if face_result.result.as_array().iter().len() > 0 {
          println!("json {}",face_result.result[0]["body"]["face_id"]);
        }
          
            // request::fetch_url("a".to_string(), file_name.to_string()).await.unwrap();            
    }
    else if metadata_class.contains("Human") {
      
    }
  
    else if metadata_class.contains("Vehicle") {
        
    }
    // 자동차 번호판일 때 차량번호 판독 모듈 실행
    else if metadata_class.contains("LicensePlate") {
    
    }      
  }  

  Ok(())
}

async fn save_bestshot(img_save_path:String, ip:String, http_port:String, image_ref:String) -> Result<String, Error> {
  let file_path = format!("{}/{}",img_save_path, ip);
  let file_name = format!("{}/{:?}.jpg", file_path.clone(), Uuid::new_v4());
  let return_file_name = file_name.to_owned();
  // tokio::spawn(async move {        

  fs::create_dir_all(file_path.clone()).unwrap();        
  let url = format!("http://{}:{}{}", ip, http_port, image_ref);
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
  
  Ok(return_file_name)
}    