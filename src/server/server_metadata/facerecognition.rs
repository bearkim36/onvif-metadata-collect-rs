use anyhow::Error;
use reqwest::{multipart, Body, Client};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::util;

pub async fn recog(filePath:String, face_recognition_url:String) -> Result<(String), Error>  {
  let client = Client::new();
  let file = File::open(filePath).await?;

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
      .part("value", some_file);
  
  //send request
  let url = format!("{}/lunaApi/createFace",face_recognition_url);
  let response = client.post(url).multipart(form).send().await?;
  let result = response.text().await?;

  Ok(result)  
}