use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

use protobuf::Message;

use serde_json::Value as JSONValue;

use geobuf::geobuf_pb::Data;

pub fn read_json_file(file_path: &str) -> JSONValue {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            println!("Could not open {}", file_path);
            process::exit(1);
        }
    };
    let buff_reader = BufReader::new(file);
    match serde_json::from_reader(buff_reader) {
        Ok(geojson) => geojson,
        Err(_) => {
            println!("Could not parse geojson: {}", file_path);
            process::exit(1);
        }
    }
}

pub fn read_pbf_file(file_path: &str) -> Data {
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(_) => {
            println!("Could not open {}", file_path);
            process::exit(1);
        }
    };
    let mut contents = vec![];
    file.read_to_end(&mut contents).unwrap();
    let mut data = Data::new();
    data.merge_from_bytes(&contents).unwrap();
    data
}
