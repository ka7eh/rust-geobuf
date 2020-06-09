extern crate geobuf;

use std::fs;
use std::io::prelude::*;
use std::process;

use clap::{crate_version, App, AppSettings, Arg};

use protobuf::Message;

use serde_json;

mod utils;

fn main() {
    let matches = App::new("geobuf")
        .about("A geobuf encoder and decoder in rust")
        .version(crate_version!())
        .setting(AppSettings::ArgRequiredElseHelp)
        .subcommand(
            App::new("encode")
                .version(crate_version!())
                .about("Convert GeoJSON file to Geobuf")
                .arg(
                    Arg::with_name("input")
                        .short('i')
                        .long("input")
                        .required(true)
                        .takes_value(true)
                        .about("Path to the input GeoJSON file"),
                )
                .arg(
                    Arg::with_name("output")
                        .short('o')
                        .long("output")
                        .required(true)
                        .takes_value(true)
                        .about("Path to the output PBF file"),
                )
                .arg(
                    Arg::with_name("dim")
                        .short('d')
                        .long("dim")
                        .takes_value(true)
                        .default_value("2")
                        .about("number of dimensions in coordinates"),
                )
                .arg(
                    Arg::with_name("precision")
                        .short('p')
                        .long("precision")
                        .takes_value(true)
                        .default_value("6")
                        .about("max number of digits after the decimal point in coordinates"),
                ),
        )
        .subcommand(
            App::new("decode")
                .version(crate_version!())
                .about("Convert Geobuf file to GeoJSON")
                .arg(
                    Arg::with_name("input")
                        .short('i')
                        .long("input")
                        .required(true)
                        .takes_value(true)
                        .about("Path to the input PBF file"),
                )
                .arg(
                    Arg::with_name("output")
                        .short('o')
                        .long("output")
                        .required(true)
                        .takes_value(true)
                        .about("Path to the output GeoJSON file"),
                )
                .arg(
                    Arg::with_name("pretty")
                        .short('p')
                        .long("pretty")
                        .about("Pretty write GeoJSON"),
                ),
        )
        .get_matches();
    let (cmd, args) = matches.subcommand();
    let (input, output, precision, dim, pretty) = match args {
        Some(v) => {
            let (precision, dim, pretty) = if cmd == "encode" {
                (v.value_of("precision"), v.value_of("dim"), false)
            } else {
                (None, None, v.occurrences_of("pretty") != 0)
            };
            (
                v.value_of("input").unwrap(),
                v.value_of("output").unwrap(),
                precision,
                dim,
                pretty,
            )
        }
        None => {
            process::exit(1);
        }
    };

    match cmd {
        "decode" => {
            let data = utils::read_pbf_file(input);
            let geojson = geobuf::decode::Decoder::decode(&data).unwrap();
            let mut f = fs::File::create(output).unwrap();
            let geojson_str = if pretty {
                serde_json::to_vec_pretty(&geojson).unwrap()
            } else {
                serde_json::to_vec(&geojson).unwrap()
            };
            f.write_all(&geojson_str).unwrap();
        }
        "encode" => {
            let geojson = utils::read_json_file(input);
            let data = geobuf::encode::Encoder::encode(
                &geojson,
                precision.unwrap().parse::<u32>().unwrap(),
                dim.unwrap().parse::<u32>().unwrap(),
            )
            .unwrap();
            let msg = data.write_to_bytes().unwrap();
            let mut f = fs::File::create(output).unwrap();
            f.write_all(&msg).unwrap();
        }
        _ => {}
    }
}
