use anyhow::Error;
use reqwest::{multipart, Body, Client};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};
use serde_json::{Value,json};
use serde::Deserialize;

use crate::util;

#[derive(Debug)]
#[derive(Deserialize)]
pub struct FaceResult {
    pub result: Value,
}

pub async fn recog(file_path:String, face_recognition_url:String) -> Result<FaceResult, Error>  {
  let client = Client::new();  
  let file = File::open(file_path).await?;  

  // read file body stream
  let stream = FramedRead::new(file, BytesCodec::new());
  let file_body = Body::wrap_stream(stream);

  //make form part of file
  let some_file = multipart::Part::stream(file_body)
      .file_name("image.jpg")
      .mime_str("image/jpeg")?;

  //create the multipart form
  let form = multipart::Form::new()
      .text("options", "image.jpg")      
      .part("file", some_file);
  
  //send request
  let url = format!("{}/lunaApi/createFace",face_recognition_url);
  let response = client.post(url).multipart(form).send().await?;  
  let json_value = response.json::<FaceResult>().await?;  

  Ok(json_value)  
}