//! Geobuf to GeoJSON decoder
use serde_json::Value as JSONValue;

use crate::geobuf_pb::{
    Data, Data_Feature, Data_FeatureCollection, Data_Feature_oneof_id_type, Data_Geometry,
    Data_Geometry_Type, Data_Value, Data_Value_oneof_value_type, Data_oneof_data_type,
};

/// Geobuf to GeoJSON Decoder
pub struct Decoder<'a> {
    data: &'a Data,
    dim: usize,
    e: f64, // multiplier for converting coordinates into integers
}

impl<'a> Decoder<'a> {
    /// Returns a GeoJSON object from the given `geobuf_pb::Data` object
    ///
    /// # Arguments
    ///
    /// * `data` - A `geobuf_pb::Data` object.
    ///
    /// # Example
    ///
    /// ```
    /// use geobuf::decode::Decoder;
    /// use geobuf::geobuf_pb;
    ///
    /// let mut data = geobuf_pb::Data::new();
    /// let feature_collection = geobuf_pb::Data_FeatureCollection::new();
    /// data.set_feature_collection(feature_collection);
    /// let geojson = Decoder::decode(&data).unwrap();
    /// assert_eq!(geojson["type"], "FeatureCollection");
    /// ```
    pub fn decode(data: &Data) -> Result<JSONValue, &'static str> {
        let dim = data.get_dimensions() as usize;
        let precision = data.get_precision() as i32;

        let decoder = Decoder {
            data,
            dim,
            e: 10f64.powi(precision),
        };

        let data_type = match decoder.data.data_type.as_ref() {
            Some(data_type) => data_type,
            None => return Err("Missing data type."),
        };

        match data_type {
            Data_oneof_data_type::feature_collection(feature_collection) => {
                Ok(decoder.decode_feature_collection(&feature_collection))
            }
            Data_oneof_data_type::feature(feature) => Ok(decoder.decode_feature(&feature)),
            Data_oneof_data_type::geometry(geometry) => Ok(decoder.decode_geometry(&geometry)),
        }
    }

    fn decode_feature_collection(&self, feature_collection: &Data_FeatureCollection) -> JSONValue {
        let mut features_json = Vec::new();
        for feature in feature_collection.get_features().iter() {
            features_json.push(self.decode_feature(feature));
        }

        let mut feature_collection_json =
            serde_json::json!({"type": "FeatureCollection", "features": features_json});

        self.decode_properties(
            &feature_collection.get_custom_properties(),
            &feature_collection.get_values(),
            &mut feature_collection_json,
        );
        feature_collection_json
    }

    fn decode_feature(&self, feature: &Data_Feature) -> JSONValue {
        let mut feature_json = serde_json::json!({
            "type": "Feature",
            "geometry": self.decode_geometry(&feature.get_geometry())
        });

        self.decode_properties(
            &feature.get_custom_properties(),
            &feature.get_values(),
            &mut feature_json,
        );

        match &feature.id_type {
            Some(id) => match id {
                Data_Feature_oneof_id_type::int_id(id) => {
                    feature_json["id"] = serde_json::json!(id)
                }
                Data_Feature_oneof_id_type::id(id) => feature_json["id"] = serde_json::json!(id),
            },
            None => {}
        }

        let feature_properties = feature.get_properties();
        if feature_properties.len() > 0 {
            let mut properties = serde_json::json!({});
            self.decode_properties(&feature_properties, &feature.get_values(), &mut properties);
            feature_json["properties"] = properties;
        }

        feature_json
    }

    fn decode_geometry(&self, geometry: &Data_Geometry) -> JSONValue {
        let mut geometry_json = serde_json::json!({});

        match geometry.get_field_type() {
            Data_Geometry_Type::GEOMETRYCOLLECTION => {
                geometry_json["type"] = serde_json::json!("GeometryCollection");
                let mut geometries = Vec::new();
                for geom in geometry.get_geometries() {
                    geometries.push(self.decode_geometry(&geom));
                }
                geometry_json["geometries"] = serde_json::json!(geometries);
            }
            Data_Geometry_Type::POINT => {
                geometry_json["type"] = serde_json::json!("Point");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_point(&geometry.get_coords()));
            }
            Data_Geometry_Type::MULTIPOINT => {
                geometry_json["type"] = serde_json::json!("MultiPoint");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_line(&geometry.get_coords(), false));
            }
            Data_Geometry_Type::LINESTRING => {
                geometry_json["type"] = serde_json::json!("LineString");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_line(&geometry.get_coords(), false));
            }
            Data_Geometry_Type::MULTILINESTRING => {
                geometry_json["type"] = serde_json::json!("MultiLineString");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_multi_line(&geometry, false));
            }
            Data_Geometry_Type::POLYGON => {
                geometry_json["type"] = serde_json::json!("Polygon");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_multi_line(&geometry, true));
            }
            Data_Geometry_Type::MULTIPOLYGON => {
                geometry_json["type"] = serde_json::json!("MultiPolygon");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_multi_polygon(&geometry));
            }
        }

        self.decode_properties(
            &geometry.get_custom_properties(),
            geometry.get_values(),
            &mut geometry_json,
        );
        geometry_json
    }

    fn decode_properties(&self, properties: &[u32], values: &[Data_Value], json: &mut JSONValue) {
        let keys = self.data.get_keys();
        for i in (0..properties.len()).step_by(2) {
            let key = &keys[properties[i] as usize];
            let value = &values[properties[i + 1] as usize];

            match value.value_type.as_ref().unwrap() {
                Data_Value_oneof_value_type::string_value(v) => json[key] = serde_json::json!(v),
                Data_Value_oneof_value_type::double_value(v) => json[key] = serde_json::json!(v),
                Data_Value_oneof_value_type::pos_int_value(v) => json[key] = serde_json::json!(v),
                Data_Value_oneof_value_type::neg_int_value(v) => {
                    json[key] = serde_json::json!(-(*v as i64))
                }
                Data_Value_oneof_value_type::bool_value(v) => json[key] = serde_json::json!(v),
                Data_Value_oneof_value_type::json_value(v) => {
                    json[key] = serde_json::from_str(v).unwrap()
                }
            }
        }
    }

    fn decode_coord(&self, coord: &i64) -> f64 {
        *coord as f64 / self.e
    }

    fn decode_point(&self, coords: &[i64]) -> Vec<f64> {
        coords
            .iter()
            .map(|coord| self.decode_coord(coord))
            .collect()
    }

    fn decode_line(&self, coords: &[i64], is_closed: bool) -> Vec<Vec<f64>> {
        let mut points_json = Vec::new();
        let mut p0 = vec![0; self.dim];

        for i in (0..coords.len()).step_by(self.dim) {
            let mut p = Vec::with_capacity(self.dim);
            let mut point = Vec::with_capacity(self.dim);
            for j in 0..self.dim {
                let coord = p0[j] + coords[i + j];
                p.push(coord);
                point.push(self.decode_coord(&coord));
            }
            points_json.push(point);
            p0 = p;
        }

        if is_closed {
            let mut p = vec![0.0; self.dim];
            for j in 0..self.dim {
                p[j] = self.decode_coord(&coords[j]);
            }
            points_json.push(p);
        }

        points_json
    }

    fn decode_multi_line(&self, geometry: &Data_Geometry, is_closed: bool) -> Vec<Vec<Vec<f64>>> {
        let lengths = geometry.get_lengths();
        let coords = geometry.get_coords();
        if lengths.len() == 0 {
            return vec![self.decode_line(&coords, is_closed)];
        }
        let mut lines = Vec::new();
        let mut i: usize = 0;

        for l in lengths {
            let end = (*l as usize) * self.dim;
            let coords = &coords[i..i + end];
            lines.push(self.decode_line(coords, is_closed));
            i += end;
        }

        lines
    }

    fn decode_multi_polygon(&self, geometry: &Data_Geometry) -> Vec<Vec<Vec<Vec<f64>>>> {
        let lengths = geometry.get_lengths();
        if lengths.len() == 0 {
            return vec![vec![self.decode_line(&geometry.get_coords(), true)]];
        }

        let mut polygons = Vec::new();
        let mut i = 0;
        let mut j = 1;
        let num_polygons = lengths[0];

        let coords = geometry.get_coords();
        for _n in 0..num_polygons {
            let num_rings = lengths[j] as usize;
            j += 1;
            let mut rings = Vec::new();
            for l in &lengths[j..j + num_rings] {
                let end = (*l as usize) * self.dim;
                let coords = &coords[i..i + end];
                rings.push(self.decode_line(coords, true));
                j += 1;
                i += end;
            }
            polygons.push(rings);
        }

        polygons
    }
}
