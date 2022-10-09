use std::sync::Arc;
use futures::StreamExt;
use url::{Url};
use anyhow::{anyhow, Error};

use retina::codec::CodecItem;
use retina::client::{SessionGroup, SetupOptions};
use async_trait::async_trait;
use uuid::Uuid;


#[async_trait]
pub trait MetadataManager {
  async fn run_onvif(&self) -> Result<(), Error> ;
}

pub struct Metadata {  
  pub url: String,  
  pub username: String,  
  pub password: String,
  pub fclt_name: String,
}


#[async_trait]
impl MetadataManager for Metadata {  
  async fn run_onvif(&self) -> Result<(), Error> {
    println!("onvif thread activation [{}]", self.fclt_name.clone());
    println!("onvif url: {}", self.url.clone());
    println!("onvif id/pw {}/{}", self.username.clone(), self.password.clone());    
    let session_group = Arc::new(SessionGroup::default());
    let creds = Some(retina::client::Credentials {
        username : self.username.clone(),
        password: self.password.clone(),
    });
    let mut session = retina::client::Session::describe(
        Url::parse(self.url.as_str())?,
        retina::client::SessionOptions::default()
            .creds(creds)
            .user_agent("Retina metadata example".to_owned())
            .session_group(session_group),
    )
    .await?;
    let onvif_stream_i = session
        .streams()
        .iter()
        .position(|s| {
            matches!(
                s.parameters(),
                Some(retina::codec::ParametersRef::Message(..))
            )
        })
        .ok_or_else(|| anyhow!("couldn't find onvif stream"))?;
    session
        .setup(onvif_stream_i, SetupOptions::default())
        .await?;
    let mut session = session
        .play(retina::client::PlayOptions::default().ignore_zero_seq(true))
        .await?
        .demuxed()?;

 
    loop {
        tokio::select! {
            item = session.next() => {
                match item.ok_or_else(|| anyhow!("EOF"))?? {
                    CodecItem::MessageFrame(m) => {

                    //    println!(
                    //         "{}: {}\n",
                    //         &m.timestamp(),
                    //         std::str::from_utf8(m.data()).unwrap(),
                    //     );
                        
                    },
                    _ => continue,
                };
            },
        }
    }
    
  }
}
