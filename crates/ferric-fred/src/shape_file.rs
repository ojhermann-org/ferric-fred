use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A GeoFRED / Maps shape file (the `geofred/shapes/file` endpoint) — the region
/// boundary polygons for a [`ShapeType`](crate::ShapeType), as a GeoJSON
/// `FeatureCollection`.
///
/// This crate **transports** GeoJSON without interpreting it (ADR-0025):
/// per-feature `properties` vary by shape type and the geometry coordinates are
/// in GeoFRED's own display projection (integer pixel-like pairs, not lat/lon),
/// so those parts are carried as [`serde_json::Value`] rather than modelled down
/// to the polygon. A consumer that needs typed, projected geometry can re-parse
/// the geometry with a dedicated GeoJSON crate. The `type`/`crs` fields are kept
/// so the value round-trips as valid GeoJSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct ShapeFile {
    /// The GeoJSON object type — `"FeatureCollection"`.
    #[serde(rename = "type")]
    pub kind: String,

    /// FRED's name for the shape set, e.g. `"state_bea_region"`.
    pub name: String,

    /// The GeoJSON coordinate reference system, carried verbatim (kept for a
    /// faithful round-trip; absent for shapes that omit it).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crs: Option<Value>,

    /// The region features — one boundary per region.
    pub features: Vec<Feature>,
}

/// A single GeoJSON feature in a [`ShapeFile`]: a region's properties and its
/// boundary geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Feature {
    /// The GeoJSON object type — `"Feature"`.
    #[serde(rename = "type")]
    pub kind: String,

    /// The region's properties, carried untyped. Keys vary by shape type (e.g. a
    /// `bea` shape carries `bea_region`/`bea_regi_1`), and FRED emits an empty
    /// **array** `[]` — not `{}` — for a feature with no properties, so this is a
    /// [`serde_json::Value`] rather than a map.
    pub properties: Value,

    /// The region's boundary geometry.
    pub geometry: Geometry,
}

/// A GeoJSON geometry within a [`Feature`]. The coordinates are carried untyped
/// (they are deep nested arrays in GeoFRED's display projection); `kind` is the
/// GeoJSON geometry type, e.g. `"MultiPolygon"`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Geometry {
    /// The GeoJSON geometry type, e.g. `"MultiPolygon"` or `"Polygon"`.
    #[serde(rename = "type")]
    pub kind: String,

    /// The geometry coordinates, verbatim — nested arrays in GeoFRED's display
    /// projection, not interpreted by this crate.
    pub coordinates: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHAPE_FILE: &str = r#"{
        "type": "FeatureCollection",
        "name": "state_bea_region",
        "crs": {"type": "name", "properties": {"name": "urn:ogc:def:crs:OGC:1.3:CRS84"}},
        "features": [
            {
                "type": "Feature",
                "properties": {"bea_region": 8, "bea_regi_1": "Far West"},
                "geometry": {"type": "MultiPolygon", "coordinates": [[[[1485, 2651], [1482, 2635]]]]}
            }
        ]
    }"#;

    #[test]
    fn parses_feature_collection() {
        let shapes: ShapeFile = serde_json::from_str(SHAPE_FILE).expect("shape file parses");
        assert_eq!(shapes.kind, "FeatureCollection");
        assert_eq!(shapes.name, "state_bea_region");
        assert_eq!(shapes.features.len(), 1);
        let feature = &shapes.features[0];
        assert_eq!(feature.kind, "Feature");
        assert_eq!(feature.properties["bea_regi_1"], Value::from("Far West"));
        assert_eq!(feature.geometry.kind, "MultiPolygon");
    }

    #[test]
    fn empty_properties_array_parses() {
        // FRED emits `[]` (not `{}`) for a feature with no properties, and can
        // return non-polygon geometries like MultiLineString — both must parse.
        let feature: Feature = serde_json::from_str(
            r#"{"type":"Feature","properties":[],
                "geometry":{"type":"MultiLineString","coordinates":[[[-707,5188],[3651,2950]]]}}"#,
        )
        .expect("empty-properties feature parses");
        assert_eq!(feature.properties, Value::Array(vec![]));
        assert_eq!(feature.geometry.kind, "MultiLineString");
    }

    #[test]
    fn round_trips_geometry_verbatim() {
        // The untyped geometry must survive a parse -> serialize cycle unchanged.
        let shapes: ShapeFile = serde_json::from_str(SHAPE_FILE).unwrap();
        let reserialized = serde_json::to_value(&shapes).unwrap();
        let original: Value = serde_json::from_str(SHAPE_FILE).unwrap();
        assert_eq!(reserialized, original);
    }
}
