use mongodb::{error::Error, results::InsertOneResult, sync::Collection};

#[derive(Clone)]
pub struct Fclt {
    collection: mongodb::sync::Collection<bson::Document>,
}

impl Fclt {
    pub fn new(collection: Collection<bson::Document>) -> Fclt {
      Fclt { collection }
    }    

    pub fn getFclt(&self) -> Result<Option<bson::Document>, Error> {
        self.collection.find(bson::doc! {}, None)
    }
}