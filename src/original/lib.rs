
use std::{thread, time, env, io, string};
use std::path::PathBuf;
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use futures::StreamExt;
use url::{Url};
use anyhow::{anyhow};

use retina::codec::CodecItem;
use retina::client::{SessionGroup, SetupOptions};

use neon::prelude::*;

use tokio::runtime::Runtime;
use std::sync::mpsc;


extern crate dotenv;

 fn add_metadata_connect(mut cx: FunctionContext)  -> JsResult<JsUndefined> {  
  // let n = cx.argument::<JsNumber>(0)?.value(&mut cx);
  // let callback = cx.argument::<JsFunction>(0).unwrap().root(&mut cx);
  // let channel = Arc::new(cx.channel());
  let (tx, rx) = std::sync::mpsc::channel();


  let mut rt = Runtime::new().unwrap();  
  let local = tokio::task::LocalSet::new();
  rt.spawn( async move {                          
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
                        let msg = String::from(std::str::from_utf8(m.data()).unwrap());                        
                        tx.send( msg).unwrap();

                        
                      },
                      _ => continue,
                  };
              },
          }
      }               
  });       
  loop {
      // Run the local task set.
    local.block_on(&rt, async {
        let callback = cx.argument::<JsFunction>(0).unwrap().root(&mut cx);
        let channel = Arc::new(cx.channel());  
      
        let message = rx.recv().unwrap();
        channel.send( move |mut cx| {
          callback
          .into_inner(&mut cx)
          .call_with(&cx)
          .arg(cx.string(message))
          .exec(&mut cx)
        });  
      
    });
  }
  Ok(cx.undefined())
}


fn get_env_path() -> io::Result<PathBuf> {
  let mut dir = env::current_exe()?;
  dir.pop();    
  dir.push(".env");
  Ok(dir)
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  cx.export_function("add_metadata_connect", add_metadata_connect)?;

  Ok(()) 
}
