extern crate winapi;

extern crate libloading;
use libloading::{Library, Symbol};


use std::os::raw::{c_char};
use std::ffi::{CString, CStr};
use std::str::Utf8Error;

const TARGET_DIR: Option<&'static str> = option_env!("CARGO_TARGET_DIR");
const TARGET_TMPDIR: Option<&'static str> = option_env!("CARGO_TARGET_TMPDIR");

fn lib_path() -> std::path::PathBuf {
  [
      TARGET_TMPDIR.unwrap_or(TARGET_DIR.unwrap_or("target")),
      "tsanpr/tsanpr.dll",
  ]
  .iter()
  .collect()
}


#[cfg(target_arch="x86_64")]
fn load_ordinal_lib() -> Library {
    unsafe {
      Library::new("tsanpr/tsanpr.dll").expect("tsanpr.dll")
    }
}


pub fn lpr_init() -> Result<&'static str, Utf8Error> {
  let lib = load_ordinal_lib();
  let func: libloading::Symbol<unsafe extern fn(*const c_char) -> *const c_char>  =   unsafe{lib.get(b"anpr_initialize").unwrap()};

  let output_parameter_ptr = CString::new("json").unwrap();
  let output_parameter: *const c_char = output_parameter_ptr.as_ptr() as *const c_char;

  let result_ptr = unsafe{ func(output_parameter) };
  let c_str = unsafe {CStr::from_ptr(result_ptr)};
  let result = c_str.to_str();

  result
}

pub fn anpr_read_file() -> Result<&'static str, Utf8Error> {
  let lib = load_ordinal_lib();
  let func: libloading::Symbol<unsafe extern fn(*const c_char, *const c_char, *const c_char) -> *const c_char>  =   unsafe{lib.get(b"anpr_read_file").unwrap()};

  let image_file_name_ptr = CString::new("D:/dk-workspace/dk-metadata-collect-server/target/debug/carplate/test.jpg").unwrap();
  let image_file_name: *const c_char = image_file_name_ptr.as_ptr() as *const c_char;

  let output_parameter_ptr = CString::new("json").unwrap();
  let output_parameter: *const c_char = output_parameter_ptr.as_ptr() as *const c_char;
  
  let option_ptr = CString::new("").unwrap();
  let option: *const c_char = option_ptr.as_ptr() as *const c_char;

  let result_ptr = unsafe{ func(image_file_name, output_parameter, option) };
  let c_str = unsafe {CStr::from_ptr(result_ptr)};
  let result = c_str.to_str();

  result
}