use crate::utils::{
    file_binary_search, get_next_ip_from_list, get_next_ip_from_u32, ip_string_to_number,
    item_binary_search,
};
use config::{Config, File, FileFormat};
// Check async version moka::future::Cache
use moka::sync::Cache;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, prelude::*, BufReader, SeekFrom};
use std::path::Path;

lazy_static! {
    #[derive(Debug)]
    static ref CONFIGURATION: HashMap<String, usize> = {
        let config_builder = Config::builder().add_source(File::new("params", FileFormat::Json));

        let config = config_builder
            .build()
            .expect("Failed to load the internal library configuration");

        config.try_deserialize::<HashMap<String, usize>>().unwrap()
    };

    static ref CACHE: Cache<String, IpInfo> = {
        Cache::new(10_000)
    };
}

// TODO: check number type
#[derive(Deserialize, Debug)]
struct LocationRecord(String, String, String, u32, String, String);

#[derive(Deserialize, Copy, Clone, Debug)]
pub struct IpBlockRecord(pub u32, pub Option<u32>, pub f32, pub f32, pub u16);

static ROOT: &str = "../data";

#[derive(Debug, PartialEq, Clone)]
pub struct IpInfo {
    pub range: (u32, u32),
    pub country: String,
    pub region: String,
    // TODO: check if possible transform to a boolean
    pub eu: String, // "1" | "0"
    pub timezone: String,
    pub city: String,
    pub ll: (f32, f32),
    pub metro: u32,
    pub area: u16,
}

impl IpInfo {
    pub async fn lookup4(ipv4: &str) -> io::Result<Self> {
        let ip = ip_string_to_number(ipv4);

        let mut next_ip = ip_string_to_number("255.255.255.255".into());

        if CACHE.get(&ipv4.to_owned()).is_some() {
            return Ok(CACHE
                .get(&ipv4.to_owned())
                .expect("Failed to read from cache."));
        }

        match read_file::<u32>("index.json").await {
            Ok(file) => {
                let root_index: isize = file_binary_search(&file, ip);

                if root_index == -1 {
                    panic!("Ip not found in the database")
                }

                next_ip = get_next_ip_from_u32(&file, root_index, next_ip);

                match read_file::<u32>(&format!("i{}.json", &root_index)).await {
                    Ok(file) => {
                        let index = file_binary_search(&file, ip)
                            + root_index
                                * CONFIGURATION
                                    .get("NUMBER_NODES_PER_MIDINDEX")
                                    .expect("Failed to fetch internal library parameters")
                                    .clone() as isize;

                        next_ip = get_next_ip_from_u32(&file, index, next_ip);

                        match read_file::<IpBlockRecord>(&format!("{index}.json")).await {
                            Ok(file) => {
                                let index = item_binary_search(&file, ip);

                                let ip_data = file[index as usize];

                                if ip_data.1 == None {
                                    panic!("1: IP doesn't any region nor country associated");
                                };

                                next_ip = get_next_ip_from_list(&file, index, next_ip);

                                match read_location_record(ip_data.1.unwrap()).await {
                                    Ok(data) => {
                                        let result = IpInfo {
                                            range: (ip_data.0, next_ip),
                                            country: data.0,
                                            region: data.1,
                                            eu: data.5,
                                            timezone: data.4,
                                            city: data.2,
                                            ll: (ip_data.2, ip_data.3),
                                            metro: data.3,
                                            area: ip_data.4,
                                        };

                                        CACHE.insert(ipv4.to_owned(), result.to_owned());

                                        Ok(result)
                                    }
                                    _ => panic!("2: IP doesn't any region nor country associated"),
                                }
                            }
                            _ => panic!("IP doesn't any region nor country associated"),
                        }
                    }
                    _ => panic!("Failed to read the next index file."),
                }
            }
            _ => panic!("Failed to read the internal index file"),
        }
    }
}

async fn read_location_record(index: u32) -> io::Result<LocationRecord> {
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
) -> io::Result<T> {
    // TODO: read file async
    let mut file =
        std::fs::File::open(Path::new(ROOT).join(file_name)).expect("Location file not found");

    let mut reader = vec![0; lenght];

    file.seek(SeekFrom::Start(offset))
        .expect("Failed to seek test start of the search buffer.");

    file.read_exact(&mut reader)
        .expect("Failed to read the searched buffer.");

    let buffer = BufReader::new(reader.as_slice());

    // TODO: Close opened file?
    let result: T = serde_json::from_reader(buffer)
        .expect("Unable to deserialize the locations.json chunk file.");

    Ok(result)
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

#[test]
fn check_cache_updates() {
    assert!(CACHE.get(&String::from("81.22.36.183")).is_none());

    futures::executor::block_on(async { IpInfo::lookup4("81.22.36.183").await.unwrap() });

    assert!(CACHE.get(&String::from("81.22.36.183")).is_some());

    assert_eq!(
        CACHE.get(&String::from("81.22.36.183")).unwrap(),
        IpInfo {
            range: (1360405504, 1360405760),
            country: "IT".to_string(),
            region: "25".to_string(),
            eu: "1".to_string(),
            timezone: "Europe/Rome".to_string(),
            city: "Milan".to_string(),
            ll: (45.4722, 9.1922),
            metro: 0,
            area: 20
        }
    );

    futures::executor::block_on(async { IpInfo::lookup4("104.88.93.92").await.unwrap() });

    assert!(CACHE.get(&String::from("81.22.36.183")).is_some());
    assert!(CACHE.get(&String::from("104.88.93.92")).is_some());

    assert_eq!(
        CACHE.get(&String::from("81.22.36.183")).unwrap(),
        IpInfo {
            range: (1360405504, 1360405760),
            country: "IT".to_string(),
            region: "25".to_string(),
            eu: "1".to_string(),
            timezone: "Europe/Rome".to_string(),
            city: "Milan".to_string(),
            ll: (45.4722, 9.1922),
            metro: 0,
            area: 20
        }
    );

    assert_eq!(
        CACHE.get(&String::from("104.88.93.92")).unwrap(),
        IpInfo {
            range: (1750618112, 1750622208),
            country: "GB".to_string(),
            region: "ENG".to_string(),
            eu: "0".to_string(),
            timezone: "Europe/London".to_string(),
            city: "London".to_string(),
            ll: (51.5164, -0.093),
            metro: 0,
            area: 20
        }
    );
}
