use futures::TryFutureExt;
use reqwest::multipart;
use std::io::Cursor;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/**
 * 루나서버에 이벤트 전달하는 요청 리퀘스트
 */
pub fn requestGenerateEvents(file_path:String) {
    let file = multipart::Part::text("image")
    .file_name("image.jpg")
    .mime_str("image/jpeg").unwrap();

    let form = multipart::Form::new()
      // Adding just a simple text field...
      .text("username", "seanmonstar")
      // And a file...
      .part("image",file);
      
  // And finally, send the form
  let client = reqwest::Client::new();
  let resp = client
      .post("http://localhost:8080/user")
      .multipart(form)
      .send()
      .unwrap_or_else(|error| {
        panic!("occured network: {:?}", error);
    });

}
 
/**
 * Web에서 이미지 얻어오는 함수 
 */
pub async fn fetch_url(url: String, file_name: String) -> Result<()> {
    let response = reqwest::get(url).await?;
    let mut file = std::fs::File::create(file_name)?;
    let mut content =  Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    Ok(())
}

