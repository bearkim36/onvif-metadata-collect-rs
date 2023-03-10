/**
 * Copyright (C) 2022 bearkim forked by rertina 
 * 
 * 
 */

use std::{thread, time, env, io};
use std::path::PathBuf;
use anyhow::{anyhow, Error};

mod server_metadata;
mod fclt;

extern crate dotenv;

async fn server_mode() -> Result<(), Error> {
    let mongo_uri = env::var("MONGO_URI").unwrap();
    let mongo_db_name = env::var("MONGO_DB_NAME").unwrap();
    println!("MONGO_URI: {}", mongo_uri);
    println!("MONGO_DB_NAME: {}", mongo_db_name);

    let server_ip = String::from(env::var("SERVER_ADDR").unwrap_or("0.0.0.0".to_string()));
    let port : u16 = env::var("SERVER_PORT").unwrap_or("8000".to_string()).parse().unwrap();
    // let fclt_obj = fclt::FcltLib::new(mongo_uri, mongo_db_name).await;
    // fclt_obj.get_fclt().await;

    let sample_list = [
    ["172.40.14.222",	  "admin",	"hanam2022!"],
    ["172.40.7.233",	  "admin",	"hanam2022!"],
    ["172.32.89.2",	    "admin",	"hanam2022!"],
    ["172.40.18.183",	  "admin",	"0p9o8i7u^^"],
    ["172.32.89.4",	    "admin",	"hanam2022!"],
    ["172.20.90.32",	  "admin",	"1q2w3e4r5t"],
    ["172.40.1.26",	    "admin",	"hanam119^^"],
    ["172.40.12.203",	  "admin",	"hanam2022!"],
    ["172.40.18.184",	  "admin",	"1q2w3e4r5t"],
    ["172.40.1.25",	    "admin",	"hanam119^^"],
    ["172.40.18.182",	  "admin",	"1q2w3e4r5t"],
    ["172.40.7.232",	  "admin",	"hanam2022!"],
    ["172.20.90.25",	  "admin",	"1q2w3e4r5t"],
    ["172.20.90.23",	  "admin",	"1q2w3e4r5t"],
    ["172.40.14.224",	  "admin",	"hanam2022!"],
    ["172.20.90.24",	  "admin",	"1q2w3e4r5t"],
    ["172.20.90.33",	  "admin",	"1q2w3e4r5t"],
    ["172.20.90.34",	  "admin",	"1q2w3e4r5t"],
    ["172.20.90.35",	  "admin",	"1q2w3e4r5t"],
    ["172.32.89.5",	    "admin",	"hanam2022!"],
    ["172.32.86.212",	  "admin",	"1q2w3e4r5t"],
    ["172.40.12.202",	  "admin",	"hanam2022!"],
    ["172.40.18.185",	  "admin",	"1q2w3e4r5t"],
    ["172.20.90.22",	  "admin",	"1q2w3e4r5t"],
    ["172.40.12.204",	  "admin",	"hanam2022!"],
    ["172.40.14.223",	  "admin",	"hanam2022!"],
    ["172.32.89.3",	    "admin",	"hanam2022!"],
    ["172.40.2.103",	  "admin",	"hanam2022!"],
    ["172.40.3.102",	  "admin",	"hanam2022!"],
    ["172.40.11.64",	  "admin",	"hanam2022!"],
    ["172.40.3.103",	  "admin",	"hanam2022!"],
    ["172.40.11.62",	  "admin",	"hanam2022!"],
    ["172.40.2.104",	  "admin",	"hanam2022!"],
    ["172.40.3.104",	  "admin",	"hanam2022!"],
    ["172.40.5.13",	    "admin",	"hanam2022!"],
    ["172.40.5.14",	    "admin",	"hanam2022!"],
    ["172.40.6.82",	    "admin",	"hanam2022!"],
    ["172.40.11.65",	  "admin",	"hanam2022!"],
    ["172.40.6.92",	    "admin",	"hanam2022!"],
    ["172.40.6.93",	    "admin",	"hanam2022!"],
    ["172.40.11.63",	  "admin",	"hanam2022!"],
    ["172.40.2.102",	  "admin",	"hanam2022!"],
    ["172.40.5.12",	    "admin",	"hanam2022!"],
    ["172.40.14.163",	  "admin",	"hanam2022!"],
    ["172.40.14.162",	  "admin",	"hanam2022!"],
    ["172.40.14.164",	  "admin",	"hanam2022!"],
    ["172.40.16.142",	  "admin",	"hanam2022!"],
    ["172.40.16.143",	  "admin",	"hanam2022!"],
    ["172.40.16.144",	  "admin",	"hanam2022!"],
    ["172.40.16.145",	  "admin",	"hanam2022!"],
    ["172.32.86.215",	  "admin",	"1q2w3e4r5t"],
    ["172.32.86.213",	  "admin",	"1q2w3e4r5t"],
    ["172.32.87.205",	  "admin",	"1q2w3e4r5t"],
    ["172.32.86.214",	  "admin",	"1q2w3e4r5t"],
    ["172.32.87.204",	  "admin",	"1q2w3e4r5t"],
    ["172.32.87.203",	  "admin",	"1q2w3e4r5t"],
    ["172.32.87.202",	  "admin",	"1q2w3e4r5t"],
    ["172.20.96.205",	  "admin",	"1q2w3e4r5t"],
    ["172.20.96.204",	  "admin",	"1q2w3e4r5t"],
    ["172.20.96.203",	  "admin",	"1q2w3e4r5t"],
    ["172.40.18.194",	  "admin",	"0p9o8i7u^^"],
    ["172.20.96.202",	  "admin",	"1q2w3e4r5t"],
    ["172.40.18.195",	  "admin",	"0p9o8i7u^^"],
    ["172.40.18.193",	  "admin",	"0p9o8i7u^^"],
    ["172.40.18.192",	  "admin",	"0p9o8i7u^^"],
    ["172.20.91.22",	  "admin",	"hanam6400!"],
    ["172.20.91.42",	  "admin",	"hanam6400!"],
    ["172.20.91.2",	    "admin",	"hanam6400!"],
    ["172.20.91.32",	  "admin",	"hanam6400!"],
    ["172.20.91.12",	  "admin",	"hanam6400!"],
    ["172.40.5.15",	    "admin",	"j486qsf3j1"],
    ["172.32.93.12",	  "admin",	"hanam6400!"],
    ["172.40.7.234",	  "admin",	"hanam2022!"],
    ["172.40.3.174",	  "admin",	"hanam119^^"],
    ["172.40.2.105",	  "admin",	"1q2w3e4r5t^^"],
    ["172.40.2.106",	  "admin",	"1q2w3e4r5t^^"],
    ["172.40.11.66",	  "admin",	"1q2w3e4r5t^^"],
    ["172.40.6.94",	    "admin",	"1q2w3e4r5t^^"],
    ["172.40.12.205",	  "admin",	"1q2w3e4r5t^^"],
    ["172.40.20.72",	  "admin",	"hanam6400!"],
    ["172.40.20.73",	  "admin",	"hanam6400!"],
    ["172.40.20.82",	  "admin",	"hanam6400!"],
    ["172.40.20.83",	  "admin",	"hanam6400!"],
    ["172.40.20.84",	  "admin",	"hanam6400!"],
    ["172.40.20.74",	  "admin",	"hanam6400!"],
    ["172.40.20.75",	  "admin",	"hanam6400!"],
    ["172.40.19.232",	  "admin",	"hanam6400!"],
    ["172.40.19.233",	  "admin",	"hanam6400!"],
    ["172.40.19.234",	  "admin",	"hanam6400!"],
    ["172.40.19.235",	  "admin",	"hanam6400!"],
    ["172.40.20.2",	    "admin",	"hanam6400!"],
    ["172.40.20.3",	    "admin",	"hanam6400!"],
    ["172.40.20.12",	  "admin",	"hanam6400!"],
    ["172.40.20.13",	  "admin",	"hanam6400!"],
    ["172.40.20.14",	  "admin",	"hanam6400!"],
    ["172.40.20.22",	  "admin",	"hanam6400!"],
    ["172.40.20.23",	  "admin",	"hanam6400!"],
    ["172.40.20.24",	  "admin",	"hanam6400!"],
    ["172.40.20.25",	  "admin",	"hanam6400!"],
    ["172.40.20.5",	    "admin",	"hanam6400!"],
    ["172.40.20.32",	  "admin",	"hanam6400!"],
    ["172.40.20.4",	    "admin",	"hanam6400!"],
    ["172.40.20.33",	  "admin",	"hanam6400!"],
    ["172.40.20.34",	  "admin",	"hanam6400!"],
    ["172.40.20.42",	  "admin",	"hanam6400!"],
    ["172.40.20.43",	  "admin",	"hanam6400!"],
    ["172.40.20.44",	  "admin",	"hanam6400!"],
    ["172.40.20.52",	  "admin",	"hanam6400!"],
    ["172.40.20.64",	  "admin",	"hanam6400!"],
    ["172.40.20.65",	  "admin",	"hanam6400!"],
    ["172.20.91.13",	  "admin",	"hanam6400!"],
    ["172.40.20.62",	  "admin",	"hanam6400!"],
    ["172.40.20.63",	  "admin",	"hanam6400!"],
    ["172.40.20.54",	  "admin",	"hanam6400!"],
    ["172.40.20.53",	  "admin",	"hanam6400!"],
    ["172.20.91.43",	  "admin",	"hanam6400!"],
    ["172.20.91.52",	  "admin",	"hanam6400!"],
    ["172.20.91.53",	  "admin",	"hanam6400!"],
    ["172.32.93.13",	  "admin",	"hanam6400!"],
    ["172.32.93.14",	  "admin",	"hanam6400!"],
    ["172.20.91.23",	  "admin",	"hanam6400!"],
    ["172.20.91.33",	  "admin",	"hanam6400!"],
    ["172.20.91.3",	    "admin",	"hanam6400!"],
    ["172.20.91.4",	    "admin",	"hanam6400!"],
    ["172.20.91.24",	  "admin",	"hanam6400!"],
    ["172.20.91.34",	  "admin",	"hanam6400!"],
    ["172.32.93.15",	  "admin",	"hanam6400!"],
    ["172.40.20.55",	  "admin",	"hanam6400!"],
    ["172.20.91.54",	  "admin",	"hanam6400!"],
    ["172.40.8.139",	  "admin",	"p7o8u7yb79"],
    ["172.40.3.89",	    "admin",	"qd5oi9g018"],
    ["172.40.8.199",	  "admin",	"a13617thvm"],
    ["172.40.3.118",	  "admin",	"85bgldd732"],
    ["172.32.86.28",	  "admin",	"a35r608phd"],
    ["172.40.3.79",	    "admin",	"c90m8k09ih"],
    ["172.32.86.29",	  "admin",	"hanam2022!"],
    ["172.40.4.168",	  "admin",	"2k0jg427fc"],
    ["172.40.8.219",	  "admin",	"jyvc35547m"],
    ["172.40.4.169",	  "admin",	"98l3hrv4v5"],
    ["172.32.87.159",	  "admin",	"k0w0kcr029"],
    ["172.32.86.109",	  "admin",	"7734i6pqnf"],
    ["172.40.5.59",	    "admin",	"71b75uia9l"],
    ["172.40.7.29",	    "admin",	"yp0s9j3x85"],
    ["172.40.5.139",	  "admin",	"a8ph83h46w"],
    ["172.40.2.29",	    "admin",	"0wl8xu2k37"],
    ["172.40.1.38",	    "admin",	"533d9aupf5"],
    ["172.40.5.58",	    "admin",	"lh7le636d8"],
    ["172.40.11.88",	  "admin",	"fy330j34tq"],
    ["172.40.8.138",	  "admin",	"rcw5u26e32"],
    ["172.32.87.158",	  "admin",	"es4s7t249n"],
    ["172.32.86.108",	  "admin",	"82av3d5ve4"],
    ["172.40.6.229",	  "admin",	"hanam2022!"],
    ["172.40.12.149",	  "admin",	"hb25s406yi"],
    ["172.40.5.48",	    "admin",	"9ymyu2y977"],
    ["172.40.6.109",	  "admin",	"hanam2022"],
    ["172.40.3.119",	  "admin",	"08l37bng1m"],
    ["172.40.3.99",	    "admin",	"g6n8215bzs"],
    ["172.32.88.89",	  "admin",	"opf0931t2w"],
    ["172.40.6.84",	    "admin",	"x550r7n1eq"],
    ["172.40.5.169",	  "admin",	"hanam2022!"],
    ["172.40.11.89",	  "admin",	"68x6yz4v7m"],
    ["172.40.16.98",	  "admin",	"ghfc9w2654"],
    ["172.40.16.99",	  "admin",	"6mmig00d57"],
    ["172.40.6.228",	  "admin",	"hanam2022!"],
    ["172.40.1.39",	    "admin",	"2447ax1ezz"],
    ["172.40.16.109",	  "admin",	"hanam2022!"],
    ["172.31.11.16",	  "admin",	"0p9o8i7u^^zz"],
    ["172.32.88.88",	  "admin",	"f814v14nof"],
    ["172.40.16.28",	  "admin",	"hanam2022!"],
    ["172.40.16.29",	  "admin",	"hanam2022!"],
    ["172.31.8.205",	  "admin",	"0p9o8i7u^^zz"],
    ["172.40.1.69",	    "admin",	"836g4gt3rr"],
    ["172.21.104.67",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.77",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.87",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.86",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.66",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.26",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.96",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.46",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.76",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.27",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.47",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.57",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.107",	"admin",	"1q2w3e4r5t"],
    ["172.21.104.106",	"admin",	"1q2w3e4r5t"],
    ["172.21.104.117",	"admin",	"1q2w3e4r5t"],
    ["172.21.104.116",	"admin",	"1q2w3e4r5t"],
    ["172.21.104.126",	"admin",	"1q2w3e4r5t"],
    ["172.21.104.97",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.16",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.56",	  "admin",	"1q2w3e4r5t"],
    ["172.21.104.127",	"admin",	"1q2w3e4r5t"],
    ["172.21.104.17",	  "admin",	"1q2w3e4r5t"]
    ];

    let threads: Vec<_> = (0..sample_list.len())
        .map(|i| {            
            tokio::spawn(async move {
                // let rtsp_url = env::var("RTSP_URL").unwrap();
                // let rtsp_id = env::var("RTSP_ID").unwrap();
                // let rtsp_pw = env::var("RTSP_PW").unwrap();
            
                let rtsp_url = sample_list[i][0];
                let rtsp_id = sample_list[i][1];
                let rtsp_pw = sample_list[i][2];


                let metadata = server_metadata::Metadata { 
                    url: String::from(rtsp_url),
                    username: String::from(rtsp_id), 
                    password: String::from(rtsp_pw),
                };
                loop {
                    println!("{} Start ONVIF session", i);
                    server_metadata::MetadataManager::run_onvif(&metadata).await;
                    
                    thread::sleep(time::Duration::from_secs(5));                    
                }
            })
        })
        .collect();

    for handle in threads {
        tokio::join!(handle);
    //     handle.join().unwrap();
    }

    Ok(())
}


fn get_env_path() -> io::Result<PathBuf> {
    let mut dir = env::current_exe()?;
    dir.pop();    
    dir.push(".env");
    Ok(dir)
}


#[tokio::main]
async fn main() { 
    let path = get_env_path().expect("Couldn't");    
    println!("{}", path.display());
    dotenv::from_filename(path).expect("Failed to open directory");

    println!("Boot on Server Mode");
    server_mode().await;        

}

