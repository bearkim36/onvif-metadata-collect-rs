use mongodb::{bson::{doc, Bson}, bson, options::IndexOptions, Client, Collection, IndexModel};
use serde::{Deserialize, Serialize};
use futures::TryStreamExt;
use std::collections::HashMap;

#[derive(Clone)]
pub struct FcltLib {
    db : mongodb::Database
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct FcltTypeData {
    dataKey: String,
    label: String,
    value: mongodb::bson::Bson
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct FcltModel {
    fcltId: String,
    fcltName: String,
    fcltTypeData: Vec<FcltTypeData>
}

#[derive(Debug, Clone)]
pub struct FcltData {
    pub fclt_id: String,
    pub fclt_name: String,
    pub camera_ip: String,
    pub rtsp_port: String,    
    pub http_port: String,
    pub camera_id: String,
    pub camera_pass: String,
    pub ai_cam_model: String,

}

impl FcltLib {
    pub async fn new(mongo_uri: String, db_name: String) -> FcltLib {        
        let client = Client::with_uri_str(mongo_uri).await.expect("failed to connect");
        let db = client.database(&db_name);
        
        
        FcltLib { db }
    }    

    pub async fn get_fclt(&self) -> Vec<FcltData> {                
        let collection = self.db.collection::<FcltModel>("fclts");
        let response = collection.find(None, None).await.unwrap();
        
        let fclt_models: Vec<FcltModel> = response.try_collect().await.unwrap();
        
        let mut fclt_datas:Vec<FcltData> = Vec::new();        

        for i in 0..fclt_models.len() {            
            println!("{:?}", fclt_models[i].fcltId);

            let mut camera_ip:String = "".to_string();
            let mut rtsp_port:String = "".to_string();
            let mut camera_id:String = "".to_string();
            let mut camera_pass:String = "".to_string();
            let mut ai_cam_model:String = "".to_string();
            let mut http_port:String = "80".to_string();

            for j in 0..fclt_models[i].fcltTypeData.len() {                 
                if fclt_models[i].fcltTypeData[j].dataKey.to_string().contains("cameraIp") {
                    camera_ip = fclt_models[i].fcltTypeData[j].value.to_string().replace("\"", "");
                }
                else if fclt_models[i].fcltTypeData[j].dataKey.to_string().contains("rtspPort") {
                    rtsp_port = fclt_models[i].fcltTypeData[j].value.to_string().replace("\"", "");
                }                
                else if fclt_models[i].fcltTypeData[j].dataKey.to_string().contains("cameraId") {
                    camera_id = fclt_models[i].fcltTypeData[j].value.to_string().replace("\"", "");
                }
                else if fclt_models[i].fcltTypeData[j].dataKey.to_string().contains("cameraPass") {
                    camera_pass = fclt_models[i].fcltTypeData[j].value.to_string().replace("\"", "");
                }
                else if fclt_models[i].fcltTypeData[j].dataKey.to_string().contains("aiCamModel") {
                    ai_cam_model = fclt_models[i].fcltTypeData[j].value.to_string().replace("\"", "");
                }
                else if fclt_models[i].fcltTypeData[j].dataKey.to_string().contains("httpPort") {
                    http_port = fclt_models[i].fcltTypeData[j].value.to_string().replace("\"", "");
                }
            }

            let fd = FcltData {
                fclt_id: fclt_models[i].fcltId.clone(),
                fclt_name: fclt_models[i].fcltName.clone(),
                camera_ip: camera_ip,
                rtsp_port: rtsp_port,
                camera_id: camera_id,
                camera_pass: camera_pass,
                ai_cam_model: ai_cam_model,
                http_port: http_port
            };


            fclt_datas.push(fd);
        }
        println!("{:?}", fclt_datas);
        
        fclt_datas
    }
}