//! ```
//! use geobuf::{decode, encode};
//! use serde_json;
//!
//! fn main() {
//!     let original_geojson =
//!         serde_json::from_str(r#"{"type": "Point", "coordinates": [100.0, 0.0]}"#).unwrap();
//!     let geobuf = encode::Encoder::encode(&original_geojson, 6, 2).unwrap();
//!     let geojson = decode::Decoder::decode(&geobuf).unwrap();
//!     assert_eq!(original_geojson, geojson);
//! }
//! ```
pub mod decode;
pub mod encode;
pub mod geobuf_pb;

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;

    use serde_json::Value as JSONValue;

    use super::decode::Decoder;
    use super::encode::Encoder;

    const DIM: u32 = 2;
    const PRECISION: u32 = 6;
    const P: f64 = 1000000.0;

    fn compare_coordinates(coords1: &Vec<JSONValue>, coords2: &Vec<JSONValue>) {
        for (idx, coord1) in coords1.iter().enumerate() {
            let coord2 = &coords2[idx];
            if coord1.is_array() {
                compare_coordinates(&coord1.as_array().unwrap(), &coord2.as_array().unwrap());
            } else if coord1.is_f64() || coord1.is_i64() || coord1.is_u64() {
                let c1 = (coord1.as_f64().unwrap() * P).round() / P;
                let c2 = (coord2.as_f64().unwrap() * P).round() / P;
                assert_eq!(c1, c2);
            }
        }
    }

    fn compare_geojsons(obj1: &JSONValue, obj2: &JSONValue) {
        if obj1.is_object() {
            for (k, v1) in obj1.as_object().unwrap() {
                let v2 = &obj2[k];
                if k == "coordinates" {
                    compare_coordinates(&obj1[k].as_array().unwrap(), &obj2[k].as_array().unwrap());
                } else if v1.is_array() || v1.is_object() {
                    compare_geojsons(v1, v2);
                } else {
                    assert_eq!(v1, v2);
                }
            }
        } else if obj1.is_array() {
            for (idx, v1) in obj1.as_array().unwrap().iter().enumerate() {
                let v2 = &obj2[idx];
                if v1.is_array() || v1.is_object() {
                    compare_geojsons(v1, v2);
                } else {
                    assert_eq!(v1, v2);
                }
            }
        } else {
            assert_eq!(obj1, obj2);
        };
    }

    fn test_geojson(file_path: &str) {
        let file = File::open(file_path).unwrap();
        let buff_reader = BufReader::new(file);
        let original_geojson = serde_json::from_reader(buff_reader).unwrap();

        let data = Encoder::encode(&original_geojson, PRECISION, DIM).unwrap();
        let geojson = Decoder::decode(&data).unwrap();

        compare_geojsons(&original_geojson, &geojson);
    }

    #[test]
    fn test_feature() {
        test_geojson("fixtures/feature.json");
    }

    #[test]
    fn test_featurecollection() {
        test_geojson("fixtures/featurecollection.json");
    }

    #[test]
    fn test_geobuf_js_issue_62() {
        test_geojson("fixtures/geobuf-js-issue-62.json");
    }

    #[test]
    fn test_geometrycollection() {
        test_geojson("fixtures/geometrycollection.json");
    }

    #[test]
    fn test_linestring() {
        test_geojson("fixtures/linestring.json");
    }

    #[test]
    fn test_multilinestring() {
        test_geojson("fixtures/multilinestring.json");
    }

    #[test]
    fn test_multipoint() {
        test_geojson("fixtures/multipoint.json");
    }

    #[test]
    fn test_multipolygon() {
        test_geojson("fixtures/multipolygon.json");
    }

    #[test]
    fn test_point() {
        test_geojson("fixtures/point.json");
    }

    #[test]
    fn test_polygon() {
        test_geojson("fixtures/polygon.json");
    }

    #[test]
    fn test_precision() {
        test_geojson("fixtures/precision.json");
    }

    #[test]
    fn test_props() {
        test_geojson("fixtures/props.json");
    }

    #[test]
    fn test_single_multipoly() {
        test_geojson("fixtures/single-multipoly.json");
    }

    #[test]
    fn test_singlemultipolygon() {
        test_geojson("fixtures/singlemultipolygon.json");
    }

    #[test]
    fn test_us_states() {
        test_geojson("fixtures/us-states.json");
    }
}
