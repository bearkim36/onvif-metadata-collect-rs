use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Error;
use chrono::NaiveDateTime;
use serde_json::Value;
use uuid::Uuid;
use std::fs;
use reqwest::Client; 


pub async fn proc(json:Value, img_save_path:String, camera_ip:String) -> Result<(), Error> {
    let meta = json["MetaDataStream"]["VideoAnalytics"].clone();
    let date_str:String = meta["Frame"]["UtcTime"].to_string().replace("\"", "");
    
    if date_str != "null" {
        let date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
        // 객체가 한 개일때
        if meta["Frame"]["Object"].as_array().iter().len() == 0 {            
            let cloned_data = meta["Frame"]["Object"].clone();

            if !cloned_data.is_null() {                                
                proc_metadata(cloned_data, img_save_path, camera_ip).await.unwrap();
            }                
           
        }
        // 객체가 여러개일 때
        else {
            for objects in meta["Frame"]["Object"].as_array().into_iter() {
                for obj in objects.iter() {
                    let cloned_data = obj.clone();
                    proc_metadata(cloned_data, img_save_path.clone(), camera_ip.clone()).await.unwrap();         
                }
            }
        }   
    }
    Ok(())
  }

  async fn proc_metadata(metadata:Value, img_save_path:String, camera_ip:String) -> Result<(), Error> {
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
    if let Some(image_ref) = cloned_data["Appearance"].get("ImageRef") {
        let save_file_name = save_bestshot(img_save_path, camera_ip, image_ref.to_string().replace("\"", "")).await.unwrap();
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
    Ok(())
  }


  async fn save_bestshot(img_save_path:String, ip:String, image_ref:String) -> Result<String, Error> {    
    let file_path = format!("{}/{}",img_save_path, ip);
    let file_name = format!("{}/{:?}.jpg", file_path.clone(), Uuid::new_v4());
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
    
    Ok(return_file_name)
  }

