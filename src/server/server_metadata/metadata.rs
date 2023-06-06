use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize ,Debug, Clone)]
pub struct XY{
  pub x:u32,
  pub y:u32,
}

#[derive(Serialize, Deserialize ,Debug, Clone)]
pub struct Rect {
  pub top: u32,
  pub bottom: u32,
  pub left: u32,
  pub right: u32,
  pub center: XY,
  pub translate: XY,
}

#[derive(Serialize, Deserialize ,Debug, Clone)]
pub struct MetadataClass {
  pub r#type: String,
  pub likelihood: String,
}

#[derive(Serialize, Deserialize ,Debug, Clone)]
pub struct Metadata {
  pub faceId:String, 
  pub fcltId:String,
  pub rect: Rect,
  pub class: MetadataClass,
  pub currentTime:u64,  
  pub plateNumberDetecting: bool,
  pub plateUuid: String,
  pub detectStatus: String,
  pub detectType: u32,
  pub vehicleType: u32, 
  // vehicleColor
}