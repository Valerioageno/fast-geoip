#![deny(clippy::all)]

mod utils;
use async_fs;
use std::io::BufReader;
use std::path::Path;

static ROOT: &str = "../data";

#[macro_use]
extern crate napi_derive;

#[napi(object)]
pub struct IpInfo {
  pub range: Vec<u32>,
  pub country: String,
  pub region: String,
  //TODO: Check if possible on Wasm char type
  pub eu: String, // "1" | "0"
  pub timezone: String,
  pub city: String,
  pub ll: Vec<f64>,
  pub metro: u32,
  pub area: u16,
}

#[napi]
async fn lookup4(ipv4: String) -> Option<IpInfo> {
  let ip = utils::ip_string_to_number(ipv4);

  let mut next_ip = utils::ip_string_to_number(String::from("255.255.255.255"));

  println!("Remove following variable");
  let info = IpInfo {
    range: vec![1360405504, 1360405760],
    country: "IT".to_string(),
    region: "25".to_string(),
    eu: "1".to_string(),
    timezone: "Europe/Rome".to_string(),
    city: "Milan".to_string(),
    ll: vec![45.4722, 9.1922],
    metro: 0,
    area: 20,
  };

  match read_file::<u32>("index.json").await {
    Ok(file) => Some(info),
    _ => None,
  }
}

async fn read_file<T: serde::de::DeserializeOwned>(file_name: &str) -> std::io::Result<Vec<T>> {
  let file = async_fs::read(Path::new(ROOT).join(file_name))
    .await
    .expect("Failed to read the file. Filename: {file_name}");

  let buffer = BufReader::new(file.as_slice());

  // TODO: maybe ::from_string() is more fast
  let json: Vec<T> = serde_json::from_reader(buffer).unwrap();

  Ok(json)
}
