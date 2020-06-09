//! Geobuf to GeoJSON decoder
use protobuf::RepeatedField;

use serde_json::Value as JSONValue;

use crate::geobuf_pb::data::feature::Id_type;
use crate::geobuf_pb::data::geometry::Type as GeometryType;
use crate::geobuf_pb::data::value::Value_type;
use crate::geobuf_pb::data::{Data_type, Feature, FeatureCollection, Geometry, Value};
use crate::geobuf_pb::Data;

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
    /// let feature_collection = geobuf_pb::data::FeatureCollection::new();
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
            Data_type::feature_collection(feature_collection) => {
                Ok(decoder.decode_feature_collection(&feature_collection))
            }
            Data_type::feature(feature) => Ok(decoder.decode_feature(&feature)),
            Data_type::geometry(geometry) => Ok(decoder.decode_geometry(&geometry)),
        }
    }

    fn decode_feature_collection(&self, feature_collection: &FeatureCollection) -> JSONValue {
        let mut features_json = Vec::new();
        for feature in feature_collection.features.iter() {
            features_json.push(self.decode_feature(feature));
        }

        let mut feature_collection_json =
            serde_json::json!({"type": "FeatureCollection", "features": features_json});

        self.decode_properties(
            &feature_collection.custom_properties,
            &feature_collection.values,
            &mut feature_collection_json,
        );
        feature_collection_json
    }

    fn decode_feature(&self, feature: &Feature) -> JSONValue {
        let mut feature_json = serde_json::json!({
            "type": "Feature",
            "geometry": self.decode_geometry(&feature.geometry.as_ref().unwrap())
        });

        self.decode_properties(
            &feature.custom_properties,
            &feature.values,
            &mut feature_json,
        );

        match &feature.id_type {
            Some(id) => match id {
                Id_type::int_id(id) => feature_json["id"] = serde_json::json!(id),
                Id_type::id(id) => feature_json["id"] = serde_json::json!(id),
            },
            None => {}
        }

        if feature.properties.len() > 0 {
            let mut properties = serde_json::json!({});
            self.decode_properties(&feature.properties, &feature.values, &mut properties);
            feature_json["properties"] = properties;
        }

        feature_json
    }

    fn decode_geometry(&self, geometry: &Geometry) -> JSONValue {
        let mut geometry_json = serde_json::json!({});

        match geometry.get_field_type() {
            GeometryType::GEOMETRYCOLLECTION => {
                geometry_json["type"] = serde_json::json!("GeometryCollection");
                let mut geometries = Vec::new();
                for geom in geometry.geometries.iter() {
                    geometries.push(self.decode_geometry(&geom));
                }
                geometry_json["geometries"] = serde_json::json!(geometries);
            }
            GeometryType::POINT => {
                geometry_json["type"] = serde_json::json!("Point");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_point(&geometry.coords));
            }
            GeometryType::MULTIPOINT => {
                geometry_json["type"] = serde_json::json!("MultiPoint");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_line(&geometry.coords, false));
            }
            GeometryType::LINESTRING => {
                geometry_json["type"] = serde_json::json!("LineString");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_line(&geometry.coords, false));
            }
            GeometryType::MULTILINESTRING => {
                geometry_json["type"] = serde_json::json!("MultiLineString");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_multi_line(&geometry, false));
            }
            GeometryType::POLYGON => {
                geometry_json["type"] = serde_json::json!("Polygon");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_multi_line(&geometry, true));
            }
            GeometryType::MULTIPOLYGON => {
                geometry_json["type"] = serde_json::json!("MultiPolygon");
                geometry_json["coordinates"] =
                    serde_json::json!(self.decode_multi_polygon(&geometry));
            }
        }

        self.decode_properties(
            &geometry.custom_properties,
            &geometry.values,
            &mut geometry_json,
        );
        geometry_json
    }

    fn decode_properties(
        &self,
        properties: &[u32],
        values: &RepeatedField<Value>,
        json: &mut JSONValue,
    ) {
        for i in (0..properties.len()).step_by(2) {
            let key = &self.data.keys[properties[i] as usize];
            let value = &values[properties[i + 1] as usize];

            match value.value_type.as_ref().unwrap() {
                Value_type::string_value(v) => json[key] = serde_json::json!(v),
                Value_type::double_value(v) => json[key] = serde_json::json!(v),
                Value_type::pos_int_value(v) => json[key] = serde_json::json!(v),
                Value_type::neg_int_value(v) => json[key] = serde_json::json!(-(*v as i64)),
                Value_type::bool_value(v) => json[key] = serde_json::json!(v),
                Value_type::json_value(v) => json[key] = serde_json::from_str(v).unwrap(),
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

    fn decode_multi_line(&self, geometry: &Geometry, is_closed: bool) -> Vec<Vec<Vec<f64>>> {
        if geometry.lengths.len() == 0 {
            return vec![self.decode_line(&geometry.coords, is_closed)];
        }
        let mut lines = Vec::new();
        let mut i: usize = 0;

        for l in &geometry.lengths {
            let end = (*l as usize) * self.dim;
            let coords = &geometry.coords[i..i + end];
            lines.push(self.decode_line(coords, is_closed));
            i += end;
        }

        lines
    }

    fn decode_multi_polygon(&self, geometry: &Geometry) -> Vec<Vec<Vec<Vec<f64>>>> {
        if geometry.lengths.len() == 0 {
            return vec![vec![self.decode_line(&geometry.coords, true)]];
        }

        let mut polygons = Vec::new();
        let mut i = 0;
        let mut j = 1;
        let num_polygons = geometry.lengths[0];

        for _n in 0..num_polygons {
            let num_rings = geometry.lengths[j] as usize;
            j += 1;
            let mut rings = Vec::new();
            for l in &geometry.lengths[j..j + num_rings] {
                let end = (*l as usize) * self.dim;
                let coords = &geometry.coords[i..i + end];
                rings.push(self.decode_line(coords, true));
                j += 1;
                i += end;
            }
            polygons.push(rings);
        }

        polygons
    }
}
