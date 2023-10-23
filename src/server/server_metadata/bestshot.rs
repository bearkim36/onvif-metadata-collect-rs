use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize ,Debug, Clone)]
pub struct Bestshot {  
  pub fclt_id:String,
  pub object_id:String,
  pub camera_ip: String,
  pub image_ref: String, 
  pub http_port: String,
  pub utc_time: String,  
  pub class: String,  
  pub date: i64,
  pub file_name: String
}