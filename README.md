# rust-geobuf

[![Crates.io](https://img.shields.io/crates/v/geobuf.svg)](https://crates.io/crates/geobuf)
[![Build Status](https://travis-ci.com/ka7eh/rust-geobuf.svg?branch=master)](https://travis-ci.com/ka7eh/rust-geobuf)
[![Coverage Status](https://coveralls.io/repos/github/ka7eh/rust-geobuf/badge.svg?branch=master)](https://coveralls.io/github/ka7eh/rust-geobuf?branch=master)

_Tested with rust 1.44_

A port of geobuf encoder and decoder into Rust and WebAssembly

## Usage

This crate provides a command line binary, a rust library, and a WebAssembly package. The binary and library are in `geobuf`
and the Rust/WebAssembly code is in `geobuf-wasm`.

### Binary:

`geobuf [encode|decode] -i <path-to-input-file> -o <path-to-output-file>`

Use `geobuf [encode|decode] --help` for more info.

### Library

```
use geobuf::{decode, encode};
use serde_json;

fn main() {
    let original_geojson = serde_json::from_str(r#"{"type": "Point", "coordinates": [100.0, 0.0]}"#).unwrap();
    let geobuf = encode::Encoder::encode(&original_geojson, 6, 2).unwrap();
    let geojson = decode::Decoder::decode(&geobuf).unwrap();
    assert_eq!(original_geojson, geojson);
}
```

### WebAssembly

The `www` folder contains a sample project showing how the wasm code can be used.

To run the example locally, clone the repo and run the following:

- `wasm-pack build -- --features wasm`
- `cd www && npm i && npm start`

> Note: The wasm code is currently slower that the [node version](https://github.com/mapbox/geobuf) and requires some refactoring to improve how data is being passed between the JS and Rust code.
