use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::process;

use clap::{Parser, Subcommand};

use protobuf::Message;

use geobuf::geobuf_pb::Data;

#[derive(Subcommand)]
enum SubCommands {
    Encode {
        #[clap(short, long, help = "Path to the input GeoJSON file")]
        input: String,

        #[clap(short, long, help = "Path to the output PBF file")]
        output: String,

        #[clap(short, long, help = "Number of dimensions in coordinates", default_value = "2")]
        dim: u32,

        #[clap(short, long, help = "Maximum number of digits after the decimal point in coordinates", default_value = "6")]
        precision: u32,
    },

    Decode {
        #[clap(short, long, help = "Path to the input PBF file")]
        input: String,

        #[clap(short, long, help = "Path to the output GeoJSON file")]
        output: String,

        #[clap(short, long, help = "Pretty write GeoJSON")]
        pretty: bool,
    }
}

#[derive(Parser, Default)]
#[clap(arg_required_else_help = true)]
#[clap(about = "Geobuf encoder and decoder")]
#[clap(version)]
struct Args {
    #[clap(subcommand)]
    commands: Option<SubCommands>
}

pub fn read_json_file(file_path: String) -> serde_json::Value {
    let file = match fs::File::open(&file_path) {
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

pub fn read_pbf_file(file_path: String) -> Data {
    let mut file = match fs::File::open(&file_path) {
        Ok(file) => file,
        Err(_) => {
            println!("Could not open {}", &file_path);
            process::exit(1);
        }
    };
    let mut contents = vec![];
    file.read_to_end(&mut contents).unwrap();
    let mut data = Data::new();
    data.merge_from_bytes(&contents).unwrap();
    data
}

fn main() {
    let matches = Args::parse();
    match matches.commands {
        Some(SubCommands::Encode { input, output, dim, precision }) => {
            let geojson = read_json_file(input);
            let data = geobuf::encode::Encoder::encode(
                &geojson,
                precision,
                dim,
            )
            .unwrap();
            let msg = data.write_to_bytes().unwrap();
            let mut f = fs::File::create(output).unwrap();
            f.write_all(&msg).unwrap();
        },
        Some(SubCommands::Decode { input, output, pretty }) => {
            let data = read_pbf_file(input);
            let geojson = geobuf::decode::Decoder::decode(&data).unwrap();
            let mut f = fs::File::create(output).unwrap();
            let geojson_str = if pretty {
                serde_json::to_vec_pretty(&geojson).unwrap()
            } else {
                serde_json::to_vec(&geojson).unwrap()
            };
            f.write_all(&geojson_str).unwrap();
        },
        None => {
            process::exit(1);
        }
    };
}
