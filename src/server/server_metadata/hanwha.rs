use anyhow::Error;
use chrono::NaiveDateTime;
use serde_json::Value;
use serde_json::json;
use uuid::Uuid;
use std::fs;
use rdkafka::producer::{FutureProducer, FutureRecord};
use reqwest::Client; 

use crate::server_metadata::metadata;
use crate::server_metadata::bestshot;

pub async fn proc(json:Value, producer:FutureProducer, fclt_id:String, camera_ip:String, http_port:String, img_save_path:String, face_recognition_url:String) -> Result<metadata::Metadata, Error> {
  let meta = json["MetadataStream"]["VideoAnalytics"].clone();
  let date_str:String = meta["Frame"]["UtcTime"].to_string().replace("\"", "");
  let mut metadata_result:metadata::Metadata = metadata::Metadata::new();

  if date_str != "null" {
      let date = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%dT%H:%M:%S%.3fZ").unwrap();
      // 객체가 한 개일때      
      let transformation_data = meta["Frame"]["Transformation"].clone();
      if meta["Frame"]["Object"].as_array().iter().len() == 0 {            
          let cloned_data = meta["Frame"]["Object"].clone();          
          if !cloned_data.is_null() {            
            metadata_result = proc_metadata(cloned_data, transformation_data, producer, fclt_id, date_str, camera_ip, http_port).await.unwrap();
          }                
         
      }
      // 객체가 여러개일 때
      else {
          for objects in meta["Frame"]["Object"].as_array().into_iter() {
              for obj in objects.iter() {
                  let cloned_data = obj.clone();   
                  metadata_result = proc_metadata(cloned_data,  transformation_data.clone(), producer.clone(), fclt_id.clone(), date_str.clone(), camera_ip.clone(), http_port.clone()).await.unwrap();
              }
          }
      }                
  }   
  Ok(metadata_result)
}

async fn proc_metadata(metadata:Value,  transformation_data:Value, producer:FutureProducer, fclt_id:String, utc_time:String, camera_ip:String, http_port:String) -> Result<metadata::Metadata, Error> {  
  let cloned_data = metadata.clone();  
  let object_id = cloned_data["ObjectId"].clone();
  let mut metadata_result:metadata::Metadata = metadata::Metadata::new();

  // let mut metadata_object = MetadataObject{
  //     object_id: object_id.to_string(),
  //     class: cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string().replace("\"", ""),
  //     object_array: json!(meta["Frame"]["Object"].clone()),
  //     last_time: date.timestamp_millis(),
  //     cross_line: vec![]
  // };
  
  // println!("{:?}", cloned_data);
  metadata_result.rect.top = cloned_data["Appearance"]["Shape"]["BoundingBox"]["top"].as_f64().unwrap();  
  metadata_result.rect.bottom = cloned_data["Appearance"]["Shape"]["BoundingBox"]["bottom"].as_f64().unwrap();
  metadata_result.rect.left = cloned_data["Appearance"]["Shape"]["BoundingBox"]["left"].as_f64().unwrap();
  metadata_result.rect.right = cloned_data["Appearance"]["Shape"]["BoundingBox"]["right"].as_f64().unwrap();
  metadata_result.rect.center.x = cloned_data["Appearance"]["Shape"]["CenterOfGravity"]["x"].as_f64().unwrap();
  metadata_result.rect.center.y = cloned_data["Appearance"]["Shape"]["CenterOfGravity"]["y"].as_f64().unwrap();
  metadata_result.rect.translate.x = transformation_data["Translate"]["x"].as_f64().unwrap();
  metadata_result.rect.translate.y = transformation_data["Translate"]["y"].as_f64().unwrap();

  let metadata_class = cloned_data["Appearance"]["Class"]["Type"]["txt"].to_string(); 
  // 메타데이터 처리
  if metadata_class.contains("Face") || metadata_class.contains("Head") {
    metadata_result.detectType = 2;            
    if (cloned_data["Appearance"].get("HumanFace")).is_some() {            
      if cloned_data["Appearance"]["HumanFace"]["Gender"].to_string().to_lowercase().contains("male") {
        metadata_result.faceGenderType = 0;
      }
      else if cloned_data["Appearance"]["HumanFace"]["Gender"].to_string().to_lowercase().contains("female") {
        metadata_result.faceGenderType = 1;
      }

      if cloned_data["Appearance"]["HumanFace"]["AgeType"].to_string().to_lowercase().contains("young") {
        metadata_result.ageType = 0;
      }
      else if cloned_data["Appearance"]["HumanFace"]["AgeType"].to_string().to_lowercase().contains("adult") {
        metadata_result.ageType = 1;
      }
      else if cloned_data["Appearance"]["HumanFace"]["AgeType"].to_string().to_lowercase().contains("middle") {
        metadata_result.ageType = 2;
      }
      else if cloned_data["Appearance"]["HumanFace"]["AgeType"].to_string().to_lowercase().contains("senior") {
        metadata_result.ageType = 3;
      }
      
      if cloned_data["Appearance"]["Accessory"]["Hat"]["Wear"].to_string().to_lowercase().contains("true") {
        metadata_result.hat = 1;
      }
      else if cloned_data["Appearance"]["Accessory"]["Hat"]["Wear"].to_string().to_lowercase().contains("false") {
        metadata_result.hat = 0;
      }
      
      if cloned_data["Appearance"]["Accessory"]["Mask"]["Wear"].to_string().to_lowercase().contains("true") {
        metadata_result.mask = 1;
      }
      else if cloned_data["Appearance"]["Accessory"]["Mask"]["Wear"].to_string().to_lowercase().contains("false") {
        metadata_result.mask = 0;
      }
      
      if cloned_data["Appearance"]["Accessory"]["Opticals"]["Wear"].to_string().to_lowercase().contains("true") {
        metadata_result.opticals = 1;
      }
      else if cloned_data["Appearance"]["Accessory"]["Opticals"]["Wear"].to_string().to_lowercase().contains("false") {
        metadata_result.opticals = 0;
      }
      
    }
  }
  else if metadata_class.contains("Human") {
    metadata_result.detectType = 0;    
    if (cloned_data["Appearance"].get("HumanBody")).is_some() {
      if cloned_data["Appearance"]["HumanBody"]["Gender"].to_string().to_lowercase().contains("male") {
        metadata_result.genderType = 0;
      }
      else if cloned_data["Appearance"]["HumanBody"]["Gender"].to_string().to_lowercase().contains("female") {
        metadata_result.genderType = 1;
      }      
    }
    if (cloned_data["Appearance"].get("Clothing")).is_some() {
      if (cloned_data["Appearance"]["Clothing"].get("Hat")).is_some() {
        if cloned_data["Appearance"]["Clothing"]["Hat"]["Wear"].to_string().to_lowercase().contains("false") {
          metadata_result.hatDetection = 0;
        }
        else if cloned_data["Appearance"]["Clothing"]["Hat"]["Wear"].to_string().to_lowercase().contains("true") {
          metadata_result.hatDetection = 1;
        } 
      }
      if (cloned_data["Appearance"]["Clothing"].get("Tops")).is_some() {      
        if cloned_data["Appearance"]["Clothing"]["Tops"]["Length"].to_string().to_lowercase().contains("short") {
          metadata_result.topLength = 0;
        }
        else if cloned_data["Appearance"]["Clothing"]["Tops"]["Length"].to_string().to_lowercase().contains("long") {
          metadata_result.topLength = 1;
        }      
      }
      if (cloned_data["Appearance"]["Clothing"].get("Bottoms")).is_some() {      
        if cloned_data["Appearance"]["Clothing"]["Bottoms"]["Length"].to_string().to_lowercase().contains("short") {
          metadata_result.bottomLength = 0;
        }
        else if cloned_data["Appearance"]["Clothing"]["Bottoms"]["Length"].to_string().to_lowercase().contains("long") {
          metadata_result.bottomLength = 1;
        }      
      }
      if (cloned_data["Appearance"]["Clothing"]["Tops"].get("Color")).is_some() && 
            (cloned_data["Appearance"]["Clothing"]["Tops"]["Color"].get("ColorCluster")).is_some() {
        for color in cloned_data["Appearance"]["Clothing"]["Tops"]["Color"]["ColorCluster"].as_array().into_iter() {
          let color_json = json!(color);          
          if color_json["ColorString"].to_string().to_lowercase().contains("black") {
            metadata_result.topColor.push(1);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("gray") {
            metadata_result.topColor.push(2);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("white") {
            metadata_result.topColor.push(3);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("red") {
            metadata_result.topColor.push(4);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("orange") {
            metadata_result.topColor.push(5);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("yellow") {
            metadata_result.topColor.push(6);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("green") {
            metadata_result.topColor.push(7);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("blue") {
            metadata_result.topColor.push(8);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("purple") {
            metadata_result.topColor.push(9);
          }
        }
      }
      if (cloned_data["Appearance"]["Clothing"]["Bottoms"].get("Color")).is_some() && 
            (cloned_data["Appearance"]["Clothing"]["Bottoms"]["Color"].get("ColorCluster")).is_some() {
        for color in cloned_data["Appearance"]["Clothing"]["Bottoms"]["Color"]["ColorCluster"].as_array().into_iter() {
          let color_json = json!(color);          
          if color_json["ColorString"].to_string().to_lowercase().contains("black") {
            metadata_result.bottomColor.push(1);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("gray") {
            metadata_result.bottomColor.push(2);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("white") {
            metadata_result.bottomColor.push(3);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("red") {
            metadata_result.bottomColor.push(4);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("orange") {
            metadata_result.bottomColor.push(5);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("yellow") {
            metadata_result.bottomColor.push(6);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("green") {
            metadata_result.bottomColor.push(7);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("blue") {
            metadata_result.bottomColor.push(8);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("purple") {
            metadata_result.bottomColor.push(9);
          }
        }
      }
    }
    if (cloned_data["Appearance"].get("Belonging")).is_some() {
      if cloned_data["Appearance"]["Belonging"]["Bag"]["Category"].to_string().contains("bag") {
        metadata_result.bagDetection = 1;
      }
      else {
        metadata_result.bagDetection = 0;
      }      
    }
    if (cloned_data["Appearance"].get("Clothing")).is_some() {
     
    }
  }
  else if metadata_class.contains("Vehicle") {
    metadata_result.detectType = 1;

    if cloned_data["Appearance"]["VehicleInfo"]["Type"].to_string().to_lowercase().contains("car") {
      metadata_result.vehicleType = 1;
    }
    else if cloned_data["Appearance"]["VehicleInfo"]["Type"].to_string().to_lowercase().contains("bus") {
      metadata_result.vehicleType = 2;
    }
    else if cloned_data["Appearance"]["VehicleInfo"]["Type"].to_string().to_lowercase().contains("truck") {
      metadata_result.vehicleType = 3;
    }
    else if cloned_data["Appearance"]["VehicleInfo"]["Type"].to_string().to_lowercase().contains("motocycle") {
      metadata_result.vehicleType = 4;
    }
    else if cloned_data["Appearance"]["VehicleInfo"]["Type"].to_string().to_lowercase().contains("bicycle") {
      metadata_result.vehicleType = 5;
    }
    

    if (cloned_data["Appearance"]["VehicleInfo"].get("Color")).is_some() && 
            (cloned_data["Appearance"]["VehicleInfo"]["Color"].get("ColorCluster")).is_some() {
        for color in cloned_data["Appearance"]["VehicleInfo"]["Color"]["ColorCluster"].as_array().into_iter() {
          let color_json = json!(color);          
          if color_json["ColorString"].to_string().to_lowercase().contains("black") {
            metadata_result.vehicleColor.push(1);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("gray") {
            metadata_result.vehicleColor.push(2);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("white") {
            metadata_result.vehicleColor.push(3);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("red") {
            metadata_result.vehicleColor.push(4);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("orange") {
            metadata_result.vehicleColor.push(5);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("yellow") {
            metadata_result.vehicleColor.push(6);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("green") {
            metadata_result.vehicleColor.push(7);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("blue") {
            metadata_result.vehicleColor.push(8);
          }
          else if color_json["ColorString"].to_string().to_lowercase().contains("purple") {
            metadata_result.vehicleColor.push(9);
          }
        }
      }
  }  
  else if metadata_class.contains("LicensePlate") {
    metadata_result.detectType = 3;
  }
  
  if let Some(image_ref) = cloned_data["Appearance"].get("ImageRef") {        
    let bestshot = bestshot::Bestshot {
      fclt_id,
      image_ref: image_ref.to_string().replace("\"", ""),
      object_id: object_id.to_string(),
      camera_ip: camera_ip.to_string(),
      http_port: http_port.to_string(),
      utc_time,
      class: metadata_class.to_string().replace("\"", "")
    };
    let bestshot_buffer = serde_json::to_string(&bestshot).expect("json serialazation failed");
    producer.send(
      FutureRecord::to("bestshot")
          .key(&object_id.to_string())
          .payload(&bestshot_buffer),
          std::time::Duration::from_secs(0)
    ).await.unwrap();         
  }
  Ok(metadata_result)
}

async fn save_bestshot(img_save_path:String, ip:String, http_port:String, image_ref:String) -> Result<String, Error> {
  let file_path = format!("{}/{}",img_save_path, ip);
  let file_name = format!("{}/{:?}.jpg", file_path.clone(), Uuid::new_v4());
  let return_file_name = file_name.to_owned();   

  fs::create_dir_all(file_path.clone()).unwrap();        
  let url = format!("http://{}:{}{}", ip, http_port, image_ref);
  let client = Client::new();                                            

  let resp = client.get(url).send().await.unwrap();

  match resp.error_for_status() {
      Ok(_res) => {
          let img_bytes = _res.bytes().await.unwrap();                                                                                        
          tokio::spawn(async move {                
            fs::write(file_name, img_bytes).expect("Unable to write file");                               
          });
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