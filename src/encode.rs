//! GeoJSON to Geobuf encoder
use protobuf::MessageField;
use serde_json::Value as JSONValue;

use crate::geobuf_pb;
// use crate::geobuf_pb::{
//     Data
// };
// use crate::geobuf_pb::data::{
//     Feature, FeatureCollection, Geometry, Value,
// };
// use crate::geobuf_pb::data::geometry;

/// GeoJSON to Geobuf encoder
pub struct Encoder {
    data: geobuf_pb::Data,
    dim: usize,
    e: f64, // multiplier for converting coordinates into integers
}

impl Encoder {
    /// Returns a Geobuf encoded object from the given geojson value
    ///
    /// # Arguments
    ///
    /// * `geojson` - A `serde_json::Value` that contains a valid geojson object.
    /// * `precision` - max number of digits after the decimal point in coordinates.
    /// * `dim` - number of dimensions in coordinates.
    ///
    /// # Example
    ///
    /// ```
    /// use geobuf::encode::Encoder;
    /// use geobuf::geobuf_pb::data::geometry::Type;
    /// use serde_json;
    ///
    /// let geojson = serde_json::from_str(r#"{"type": "Point", "coordinates": [100.0, 0.0]}"#).unwrap();
    /// let geobuf = Encoder::encode(&geojson, 6, 2).unwrap();
    /// assert_eq!(geobuf.dimensions(), 2);
    /// assert_eq!(geobuf.precision(), 6);
    /// assert_eq!(geobuf.geometry().type_(), Type::POINT);
    /// ```
    pub fn encode(
        geojson: &JSONValue,
        precision: u32,
        dim: u32,
    ) -> Result<geobuf_pb::Data, &'static str> {
        let mut data = geobuf_pb::Data::new();
        data.set_precision(precision);
        data.set_dimensions(dim);

        let mut encoder = Encoder {
            data,
            dim: dim as usize,
            e: 10f64.powi(precision as i32),
        };

        match geojson["type"].as_str().unwrap() {
            "FeatureCollection" => match encoder.encode_feature_collection(geojson) {
                Ok(fc) => encoder.data.set_feature_collection(fc),
                Err(err) => return Err(err),
            },
            "Feature" => match encoder.encode_feature(geojson) {
                Ok(f) => encoder.data.set_feature(f),
                Err(err) => return Err(err),
            },
            _ => match encoder.encode_geometry(geojson) {
                Ok(g) => encoder.data.set_geometry(g),
                Err(err) => return Err(err),
            },
        };

        Ok(encoder.data)
    }

    fn encode_feature_collection(
        &mut self,
        geojson: &JSONValue,
    ) -> Result<geobuf_pb::data::FeatureCollection, &'static str> {
        let mut feature_collection = geobuf_pb::data::FeatureCollection::new();

        let properties = self.encode_custom_properties(
            &mut feature_collection.values,
            geojson,
            vec!["type", "features"],
        );
        feature_collection.custom_properties = properties;

        let features = &mut feature_collection.features;
        for feature in geojson["features"].as_array().unwrap() {
            match self.encode_feature(feature) {
                Ok(f) => features.push(f),
                Err(err) => return Err(err),
            }
        }

        Ok(feature_collection)
    }

    fn encode_feature(
        &mut self,
        feature_json: &JSONValue,
    ) -> Result<geobuf_pb::data::Feature, &'static str> {
        let mut feature = geobuf_pb::data::Feature::new();

        match &feature_json["id"] {
            JSONValue::Number(id) => feature.set_int_id(id.as_i64().unwrap()),
            JSONValue::String(id) => feature.set_id(String::from(id)),
            _ => {}
        }

        match feature_json["properties"].as_object() {
            Some(properties_json) => {
                let mut properties: Vec<u32> = Vec::new();
                for (key, value) in properties_json.iter() {
                    self.encode_property(
                        String::from(key),
                        value,
                        &mut properties,
                        &mut feature.values,
                    );
                }
                feature.properties = properties;
            }
            None => {}
        }

        let custom_properties = self.encode_custom_properties(
            &mut feature.values,
            feature_json,
            vec!["type", "id", "properties", "geometry"],
        );

        feature.custom_properties = custom_properties;

        match self.encode_geometry(&feature_json["geometry"]) {
            Ok(g) => {
                feature.geometry = MessageField::some(g);
            }
            Err(err) => return Err(err),
        }

        Ok(feature)
    }

    fn encode_geometry(
        &mut self,
        geometry_json: &JSONValue,
    ) -> Result<geobuf_pb::data::Geometry, &'static str> {
        let mut geometry = geobuf_pb::data::Geometry::new();

        let custom_properties = self.encode_custom_properties(
            &mut geometry.values,
            geometry_json,
            vec![
                "type",
                "id",
                "coordinates",
                "arcs",
                "geometries",
                "properties",
            ],
        );

        geometry.custom_properties = custom_properties;

        match geometry_json["type"].as_str().unwrap() {
            "GeometryCollection" => {
                geometry.set_type(geobuf_pb::data::geometry::Type::GEOMETRYCOLLECTION);
                for geom_json in geometry_json["geometries"].as_array().unwrap() {
                    match self.encode_geometry(geom_json) {
                        Ok(g) => geometry.geometries.push(g),
                        Err(err) => return Err(err),
                    }
                }
            }
            "Point" => {
                geometry.set_type(geobuf_pb::data::geometry::Type::POINT);
                for coord in geometry_json["coordinates"].as_array().unwrap() {
                    self.add_coord(&mut geometry.coords, coord.as_f64().unwrap());
                }
            }
            "MultiPoint" => {
                geometry.set_type(geobuf_pb::data::geometry::Type::MULTIPOINT);
                self.add_line(
                    &mut geometry.coords,
                    geometry_json["coordinates"].as_array().unwrap(),
                    false,
                );
            }
            "LineString" => {
                geometry.set_type(geobuf_pb::data::geometry::Type::LINESTRING);
                self.add_line(
                    &mut geometry.coords,
                    geometry_json["coordinates"].as_array().unwrap(),
                    false,
                );
            }
            "MultiLineString" => {
                geometry.set_type(geobuf_pb::data::geometry::Type::MULTILINESTRING);
                self.add_multi_line(
                    &mut geometry,
                    geometry_json["coordinates"].as_array().unwrap(),
                    false,
                );
            }
            "Polygon" => {
                geometry.set_type(geobuf_pb::data::geometry::Type::POLYGON);
                self.add_multi_line(
                    &mut geometry,
                    geometry_json["coordinates"].as_array().unwrap(),
                    true,
                );
            }
            "MultiPolygon" => {
                geometry.set_type(geobuf_pb::data::geometry::Type::MULTIPOLYGON);
                self.add_multi_polygon(
                    &mut geometry,
                    geometry_json["coordinates"].as_array().unwrap(),
                );
            }
            _ => {
                return Err("Invalid geometry type");
            }
        }
        Ok(geometry)
    }

    fn encode_custom_properties(
        &mut self,
        values: &mut Vec<geobuf_pb::data::Value>,
        custom_properties_json: &JSONValue,
        exclude: Vec<&str>,
    ) -> Vec<u32> {
        let mut properties: Vec<u32> = Vec::new();
        for (key, value) in custom_properties_json.as_object().unwrap().iter() {
            if !exclude.contains(&key.as_str()) {
                self.encode_property(String::from(key), value, &mut properties, values);
            }
        }
        properties
    }

    fn encode_property(
        &mut self,
        key: String,
        value: &JSONValue,
        properties: &mut Vec<u32>,
        values: &mut Vec<geobuf_pb::data::Value>,
    ) {
        let data_keys = &mut self.data.keys;
        match data_keys.iter().position(|k| k == &key) {
            Some(key_index) => {
                properties.push(key_index as u32);
            }
            None => {
                data_keys.push(key);
                properties.push(data_keys.len() as u32 - 1);
            }
        }

        let mut data_value = geobuf_pb::data::Value::new();
        match value {
            JSONValue::String(v) => {
                data_value.set_string_value(v.clone());
                values.push(data_value);
            }
            JSONValue::Bool(v) => {
                data_value.set_bool_value(*v);
                values.push(data_value);
            }
            JSONValue::Number(v) => {
                Encoder::encode_number(&mut data_value, v);
                values.push(data_value);
            }
            JSONValue::Object(_) | JSONValue::Array(_) => {
                data_value.set_json_value(value.to_string());
                values.push(data_value);
            }
            JSONValue::Null => {}
        };
        properties.push(values.len() as u32 - 1);
    }

    fn encode_number(value: &mut geobuf_pb::data::Value, number: &serde_json::Number) {
        if number.is_u64() {
            value.set_pos_int_value(number.as_u64().unwrap());
        } else if number.is_i64() {
            value.set_neg_int_value(number.as_i64().unwrap().abs() as u64);
        } else if number.is_f64() {
            value.set_double_value(number.as_f64().unwrap());
        }
    }

    fn add_coord(&self, coords: &mut Vec<i64>, coord: f64) {
        coords.push((coord * self.e).round() as i64);
    }

    fn add_line(&self, coords: &mut Vec<i64>, points: &[JSONValue], is_closed: bool) {
        let mut sum = vec![0; self.dim];
        for point in points.iter().take(points.len() - is_closed as usize) {
            for j in 0..self.dim {
                let coord = point[j].as_f64().unwrap();
                let n = (coord * self.e).round() as i64 - sum[j];
                coords.push(n);
                sum[j] += n;
            }
        }
    }

    fn add_multi_line(
        &self,
        geometry: &mut geobuf_pb::data::Geometry,
        lines_json: &Vec<JSONValue>,
        is_closed: bool,
    ) {
        if lines_json.len() != 1 {
            for points_json in lines_json {
                let points = points_json.as_array().unwrap();
                geometry
                    .lengths
                    .push(points.len() as u32 - is_closed as u32);
                self.add_line(&mut geometry.coords, points, is_closed);
            }
        } else {
            for line_json in lines_json {
                let line = line_json.as_array().unwrap();
                self.add_line(&mut geometry.coords, line, is_closed);
            }
        }
    }

    fn add_multi_polygon(
        &self,
        geometry: &mut geobuf_pb::data::Geometry,
        polygons_json: &Vec<JSONValue>,
    ) {
        if polygons_json.len() != 1 || polygons_json[0].as_array().unwrap().len() != 1 {
            geometry.lengths.push(polygons_json.len() as u32);
            for rings_json in polygons_json {
                let rings = rings_json.as_array().unwrap();
                geometry.lengths.push(rings.len() as u32);
                for points_json in rings {
                    let points = points_json.as_array().unwrap();
                    geometry.lengths.push(points.len() as u32 - 1);
                    self.add_line(&mut geometry.coords, points, true);
                }
            }
        } else {
            for rings_json in polygons_json {
                let rings = rings_json.as_array().unwrap();
                for points_json in rings {
                    let points = points_json.as_array().unwrap();
                    self.add_line(&mut geometry.coords, points, true);
                }
            }
        }
    }
}
