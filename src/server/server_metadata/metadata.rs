use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize ,Debug, Clone)]
pub struct XY{
  pub x:f64,
  pub y:f64,
}

#[derive(Serialize, Deserialize ,Debug, Clone)]
pub struct Rect {
  pub top: f64,
  pub bottom: f64,
  pub left: f64,
  pub right: f64,
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
  // pub plateNumberDetecting: bool,
  // pub plateUuid: String,
  // pub detectStatus: String,
  pub detectType: i32,  
  // Human
  pub genderType: i32,
  pub hatDetection: i32,
  pub bagDetection: i32,
  pub topLength: i32,
  pub bottomLength: i32,
  pub topColor: Vec<u32>,
  pub bottomColor: Vec<u32>,
  // Face
  pub faceGenderType: i32,
  pub ageType: i32,
  pub hat: i32,
  pub mask: i32,
  pub opticals: i32,
  // Vehicle
  pub vehicleType: i32, 
  pub vehicleColor: Vec<u32>,
}

impl Metadata {
  pub fn new() -> Metadata {
    Metadata {       
        faceId: "".to_string(),
        fcltId: "".to_string(),
        rect: Rect {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 0.0,
            center: XY {
                x: 0.0,
                y: 0.0,
            },
            translate: XY {
              x: 0.0,
              y: 0.0,
          },
        },
        class: MetadataClass {
            r#type: "".to_string(),
            likelihood: "".to_string(),
        },
        currentTime: 0,
        // plateNumberDetecting: false,
        // plateUuid: "".to_string(),
        // detectStatus: "".to_string(),
        detectType: -1,        
        genderType: -1,
        hatDetection: -1,
        bagDetection: -1,
        topLength: -1,
        bottomLength: -1,
        topColor: vec![],
        bottomColor: vec![],
        faceGenderType: -1,
        ageType: -1,
        hat: -1,
        mask: -1,
        opticals: -1,
        vehicleType: -1,
        vehicleColor: vec![],
    }     
  }
}