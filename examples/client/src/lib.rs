
use std::{thread, time, env, io};
use std::path::PathBuf;
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use futures::StreamExt;
use url::{Url};
use anyhow::{anyhow, Error};

use retina::codec::CodecItem;
use retina::client::{SessionGroup, SetupOptions};
use async_trait::async_trait;
use uuid::Uuid;
use lazy_static::lazy_static; 
use serde_json::{Value,json,Number};

use quickxml_to_serde::{xml_string_to_json, Config,NullValue};
use chrono::*;
mod metadata;
use metadata::{Metadata, MetadataManager};

use neon::prelude::*;

extern crate dotenv;

#[neon::main]
fn main(mut cx: ModuleContext<'_>) -> NeonResult<()> {
  cx.export_function("add_metadata_connect", add_metadata_connect)?;
    Ok(())
}

 fn add_metadata_connect(mut cx: FunctionContext<'_>) -> JsResult<JsUndefined> {
  let filename = cx.argument::<JsString>(0)?.value(&mut cx);
  let callback = cx.argument::<JsFunction>(1)?.root(&mut cx);
  let channel = cx.channel();

  tokio::spawn(async move {                

      let session_group = Arc::new(SessionGroup::default());
      let creds = Some(retina::client::Credentials {
          username : String::from("admin"),
          password: String::from("!qaz2wsx"),
      });
      let mut session = retina::client::Session::describe(
          Url::parse( String::from("rtsp://118.36.97.121/profile1/media.smp").as_str()).unwrap(),
          retina::client::SessionOptions::default()
              .creds(creds)
              .user_agent("Retina metadata example".to_owned())
              .session_group(session_group),
      )
      .await.unwrap();
      let onvif_stream_i = session
        .streams()
        .iter()
        .position(|s| {
            matches!(
                s.parameters(),
                Some(retina::codec::ParametersRef::Message(..))
            )
        })
        .ok_or_else(|| anyhow!("couldn't find onvif stream")).unwrap();
          session
        .setup(onvif_stream_i, SetupOptions::default())
        .await.unwrap();
      let mut session = session
        .play(retina::client::PlayOptions::default().ignore_zero_seq(true))
        .await.unwrap()
        .demuxed().unwrap();
        loop {
          tokio::select! {
              item = session.next() => {
                  match item.ok_or_else(|| anyhow!("EOF")).unwrap() {
                      Ok(CodecItem::MessageFrame(m)) => {
                                                        
                        println!("{:?}", m);
                       
   
                      },
                      _ => continue,
                  };
              },
          }
      }
      // loop {
      //     println!("start onvif session");
      //     if let Err(_err) = metadata.run_onvif().await {
      //         println!("retry 5sec after");
      //         thread::sleep(time::Duration::from_secs(5));
      //     }
      // } 

  });         

  Ok(cx.undefined())
}


fn get_env_path() -> io::Result<PathBuf> {
  let mut dir = env::current_exe()?;
  dir.pop();    
  dir.push(".env");
  Ok(dir)
}
