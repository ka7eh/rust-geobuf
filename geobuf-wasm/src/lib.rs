use geobuf::decode::Decoder;
use geobuf::encode::Encoder;
use geobuf::geobuf_pb::Data;
use protobuf::Message;
use serde_json;
use wasm_bindgen::prelude::*;

mod utils;

/// Enables logging of errors
#[wasm_bindgen]
pub fn debug() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub fn decode(data: &[u8]) -> JsValue {
    let mut geobuf = Data::new();
    geobuf.merge_from_bytes(&data).unwrap();
    let geojson = Decoder::decode(&geobuf).unwrap();
    JsValue::from_serde(&geojson).unwrap()
}

#[wasm_bindgen]
pub fn encode(geojson_str: &str, precision: u32, dim: u32) -> Vec<u8> {
    let geojson = serde_json::from_str(geojson_str).unwrap();
    Encoder::encode(&geojson, precision, dim)
        .unwrap()
        .write_to_bytes()
        .unwrap()
}
