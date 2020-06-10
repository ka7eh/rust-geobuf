use cfg_if::cfg_if;
use protobuf::Message;
use serde_json;
use wasm_bindgen::prelude::*;

use crate::decode::Decoder;
use crate::encode::Encoder;
use crate::geobuf_pb::Data;

cfg_if! {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function to get better error messages if we ever panic.
    if #[cfg(feature = "console_error_panic_hook")] {
        extern crate console_error_panic_hook;
        pub use self::console_error_panic_hook::set_once as set_panic_hook;
    } else {
        #[inline]
        pub fn set_panic_hook() {}
    }
}

/// Enables logging of errors
#[wasm_bindgen]
pub fn debug() {
    set_panic_hook();
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
