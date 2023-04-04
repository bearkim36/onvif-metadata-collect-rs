use std::path::PathBuf;
use std::{env, io};

pub fn get_env_path() -> io::Result<PathBuf> {
  let mut dir = env::current_exe()?;
  dir.pop();    
  dir.push(".env");
  Ok(dir)
}
