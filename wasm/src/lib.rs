#![deny(clippy::all)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate napi_derive;

mod utils;
use async_fs::{self, File};
use config::{Config, FileFormat};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{prelude::*, BufReader, SeekFrom};
use std::path::Path;

static ROOT: &str = "../data";

#[derive(Deserialize, Debug)]
struct LocationRecord(String, String, String, u32, String, String);

#[derive(Deserialize, Copy, Clone, Debug)]
pub struct IpBlockRecord(pub u32, pub Option<u32>, pub f64, pub f64, pub u16);

#[napi(object)]
struct IpInfo {
  pub range: Vec<u32>,
  pub country: String,
  pub region: String,
  //TODO: Check if possible on WASM use the char type
  pub eu: String, // "1" | "0"
  pub timezone: String,
  pub city: String,
  pub ll: Vec<f64>,
  pub metro: u32,
  pub area: u16,
}

lazy_static! {
    #[derive(Debug)]
    static ref CONFIGURATION: HashMap<String, usize> = {
        let config_builder = Config::builder().add_source(File::new("params", FileFormat::Json));

        let config = config_builder
            .build()
            .expect("Failed to load the internal library configuration");

        config.try_deserialize::<HashMap<String, usize>>().unwrap()
    };
}

#[napi]
async fn lookup4(ipv4: String) -> Option<IpInfo> {
  let ip = utils::ip_string_to_number(&ipv4);

  let mut next_ip = utils::ip_string_to_number(&String::from("255.255.255.255"));

  match read_file::<u32>("index.json").await {
    Ok(file) => {
      let root_index: isize = utils::file_binary_search(&file, ip);

      if root_index == -1 {
        println!("{} not found in the database", &ipv4);
        return None;
      }

      next_ip = utils::get_next_ip_from_u32(&file, root_index, next_ip);

      match read_file::<u32>(&format!("i{}.json", &root_index)).await {
        Ok(file) => {
          let index = utils::file_binary_search(&file, ip)
            + root_index
              * *CONFIGURATION
                .get("NUMBER_NODES_PER_MIDINDEX")
                .expect("Failed to fetch internal library parameters") as isize;

          next_ip = utils::get_next_ip_from_u32(&file, index, next_ip);

          match read_file::<IpBlockRecord>(&format!("{index}.json")).await {
            Ok(file) => {
              let index = utils::item_binary_search(&file, ip);

              let ip_data = file[index as usize];

              if ip_data.1 == None {
                println!("IP doesn't any region nor country associated");
                return None;
              };

              next_ip = utils::get_next_ip_from_list(&file, index, next_ip);

              match read_location_record(ip_data.1.unwrap()).await {
                Ok(data) => {
                  let result = IpInfo {
                    range: vec![ip_data.0, next_ip],
                    country: data.0,
                    region: data.1,
                    eu: data.5,
                    timezone: data.4,
                    city: data.2,
                    ll: vec![ip_data.2, ip_data.3],
                    metro: data.3,
                    area: ip_data.4,
                  };

                  Some(result)
                }
                _ => None,
              }
            }
            _ => None,
          }
        }
        _ => None,
      }
    }
    _ => None,
  }
}

async fn read_location_record(index: u32) -> std::io::Result<LocationRecord> {
  let location_record_size = CONFIGURATION
    .get("LOCATION_RECORD_SIZE")
    .expect("Failed to read the params internal file");

  read_file_chunk::<LocationRecord>(
    "locations.json",
    ((index as usize) * location_record_size + 1) as u64,
    location_record_size - 1,
  )
  .await
}

async fn read_file_chunk<T: serde::de::DeserializeOwned>(
  file_name: &str,
  offset: u64,
  lenght: usize,
) -> std::io::Result<T> {
  // TODO: read file async. Contribute to https://crates.io/crates/async-fs
  let mut file = std::fs::File::open(Path::new(ROOT).join(file_name))
    .expect("Location file not found");

  let mut reader = vec![0; lenght];

  file
    .seek(SeekFrom::Start(offset))
    .expect("Failed to seek test start of the search buffer.");

  file
    .read_exact(&mut reader)
    .expect("Failed to read the searched buffer.");

  let buffer = BufReader::new(reader.as_slice());

  let result: T =
    serde_json::from_reader(buffer).expect("Unable to deserialize the locations.json chunk file.");

  Ok(result)
}

async fn read_file<T: serde::de::DeserializeOwned>(file_name: &str) -> std::io::Result<Vec<T>> {
  let file = async_fs::read_to_string(Path::new(ROOT).join(file_name))
    .await
    .expect("Failed to read the db file: {file_name}");

  // read from string since is faster than from_reader https://github.com/serde-rs/json/issues/160
  let json: Vec<T> = serde_json::from_str(&file[..]).unwrap();

  Ok(json)
}

