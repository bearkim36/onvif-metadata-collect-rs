use mongodb::{bson::doc, options::IndexOptions, Client , Collection, IndexModel};
use serde::{Deserialize, Serialize};


#[derive(Clone)]
pub struct FcltLib {
    db : mongodb::Database
}

#[derive(Debug, Serialize, Deserialize)]
struct FcltModel {
    fcltId: String,
    fcltName: String,
}

impl FcltLib {
    pub async fn new(mongo_uri: String, db_name: String) -> FcltLib {        
        let client = Client::with_uri_str(mongo_uri).await.expect("failed to connect");
        let db = client.database(&db_name);
        FcltLib { db }
    }    

    pub async fn get_fclt(&self)  {                
        let collection = self.db.collection::<FcltModel>("books");
        let result = collection.find(None, None).await.unwrap();        
        println!("{:?}", result);
        
    }
}