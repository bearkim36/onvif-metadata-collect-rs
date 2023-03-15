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
    value: String
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
struct FcltModel {
    fcltId: String,
    fcltName: String,
    fcltTypeData: mongodb::bson::Bson
}

impl FcltLib {
    pub async fn new(mongo_uri: String, db_name: String) -> FcltLib {        
        let client = Client::with_uri_str(mongo_uri).await.expect("failed to connect");
        let db = client.database(&db_name);
        
        
        FcltLib { db }
    }    

    pub async fn get_fclt(&self)  {                
        let collection = self.db.collection::<FcltModel>("fclts");
        let response = collection.find(None, None).await.unwrap();
        
        let fclt_models: Vec<FcltModel> = response.try_collect().await.unwrap();
        
        
        
        println!("{:?}", fclt_models[0].fcltTypeData.get(0));
        
    }
}