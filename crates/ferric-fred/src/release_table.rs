use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{ReleaseElementId, ReleaseId, SeriesId};

/// A release's table tree, from the `fred/release/tables` endpoint — the layout
/// a release uses to present its series (sections and tables, with series rows
/// nested beneath them).
///
/// FRED returns the top-level `elements` as a JSON object keyed by element id
/// whose values are the *roots* of the tree, each carrying its subtree inline
/// via [`children`](ReleaseTableElement::children). We collect those into an
/// ordered [`roots`](ReleaseTable::roots) vector (each element already carries
/// its own id). `name` and `element_id` are present only when a subtree was
/// requested (see [`ReleaseTablesRequest::element`](crate::ReleaseTablesRequest::element));
/// for a whole-release request they are absent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseTable {
    /// The name of the requested element, when a subtree was requested.
    #[serde(default)]
    pub name: Option<String>,

    /// The id of the requested element, when a subtree was requested.
    #[serde(default)]
    pub element_id: Option<ReleaseElementId>,

    /// The root elements of the tree, ordered by element id. (FRED's redundant
    /// top-level `release_id` — a string, unlike the numeric one on each
    /// element — is dropped; the caller already knows it.)
    ///
    /// On the wire FRED names this `elements` (an object keyed by id); we read
    /// that but re-serialize as a `roots` array, matching this field and the
    /// flattened shape.
    #[serde(
        rename(serialize = "roots", deserialize = "elements"),
        deserialize_with = "roots_from_map"
    )]
    pub roots: Vec<ReleaseTableElement>,
}

/// A node in a release's table tree: a section, a table, or a series row. Nodes
/// nest via [`children`](ReleaseTableElement::children) to arbitrary depth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseTableElement {
    /// This element's id.
    pub element_id: ReleaseElementId,

    /// The release this element belongs to.
    pub release_id: ReleaseId,

    /// The parent element's id, absent for a root.
    #[serde(default)]
    pub parent_id: Option<ReleaseElementId>,

    /// The series this element points to, for a `series`-type row. Absent for
    /// structural elements (sections/tables), where FRED sends `null` or `""`.
    #[serde(default, deserialize_with = "optional_series_id")]
    pub series_id: Option<SeriesId>,

    /// The element kind, e.g. `"section"`, `"table"`, or `"series"`. Kept as a
    /// string (its vocabulary is open-ended and thinly documented; ADR-0017).
    #[serde(rename = "type")]
    pub element_type: String,

    /// Human-readable label, e.g. `"CPI for U.S. City Average"`.
    pub name: String,

    /// The element's line number within its table, when FRED provides one.
    #[serde(default)]
    pub line: Option<String>,

    /// The element's depth as FRED reports it (`"0"` at the top). Mirrors the
    /// nesting of [`children`](ReleaseTableElement::children).
    pub level: String,

    /// The child elements nested beneath this one (empty for a leaf).
    #[serde(default)]
    pub children: Vec<ReleaseTableElement>,
}

/// Deserialize FRED's `elements` object (keyed by element id) into an ordered
/// vector of its values. Ordering is by element id, so the result is
/// deterministic regardless of the object's key order.
fn roots_from_map<'de, D>(deserializer: D) -> Result<Vec<ReleaseTableElement>, D::Error>
where
    D: Deserializer<'de>,
{
    // Keys are stringified ids; each value already carries its own element_id,
    // so we keep only the values and sort by that.
    let map: BTreeMap<String, ReleaseTableElement> = BTreeMap::deserialize(deserializer)?;
    let mut roots: Vec<ReleaseTableElement> = map.into_values().collect();
    roots.sort_by_key(|element| element.element_id);
    Ok(roots)
}

/// Deserialize a `series_id` that FRED sends as `null`, an empty string, or a
/// real id, mapping the first two to `None`.
fn optional_series_id<'de, D>(deserializer: D) -> Result<Option<SeriesId>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Option<String> = Option::deserialize(deserializer)?;
    Ok(raw.filter(|id| !id.is_empty()).map(SeriesId::new))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A two-level tree: a section root containing one series row, which in turn
    /// has a series child. Mirrors the real `fred/release/tables` shape (nulls
    /// for structural elements, a real series_id on a leaf).
    const TABLE_BODY: &str = r#"{
        "name": null,
        "element_id": null,
        "release_id": "10",
        "elements": {
            "34483": {
                "element_id": 34483, "release_id": 10, "parent_id": null,
                "series_id": null, "type": "section", "name": "Monthly, SA",
                "line": null, "level": "0",
                "children": [
                    {
                        "element_id": 34484, "release_id": 10, "parent_id": 34483,
                        "series_id": "", "type": "series", "name": "All items",
                        "line": "1", "level": "1",
                        "children": [
                            {
                                "element_id": 34485, "release_id": 10, "parent_id": 34484,
                                "series_id": "CPIFABSL", "type": "series",
                                "name": "Food and beverages", "line": "2", "level": "2",
                                "children": []
                            }
                        ]
                    }
                ]
            }
        }
    }"#;

    #[test]
    fn deserializes_a_nested_table() {
        let table: ReleaseTable = serde_json::from_str(TABLE_BODY).unwrap();

        // Whole-release request: no requested-element name/id.
        assert!(table.name.is_none());
        assert!(table.element_id.is_none());

        assert_eq!(table.roots.len(), 1);
        let section = &table.roots[0];
        assert_eq!(section.element_id, ReleaseElementId::new(34483));
        assert_eq!(section.element_type, "section");
        assert!(section.parent_id.is_none());
        assert!(section.series_id.is_none()); // null → None
        assert_eq!(section.children.len(), 1);

        let all_items = &section.children[0];
        assert_eq!(all_items.parent_id, Some(ReleaseElementId::new(34483)));
        assert!(all_items.series_id.is_none()); // "" → None
        assert_eq!(all_items.line.as_deref(), Some("1"));

        let leaf = &all_items.children[0];
        assert_eq!(leaf.series_id, Some(SeriesId::new("CPIFABSL")));
        assert_eq!(leaf.element_type, "series");
        assert!(leaf.children.is_empty());
    }

    #[test]
    fn roots_are_ordered_by_element_id() {
        // Object key order is largest-first; roots must come back id-ascending.
        let body = r#"{
            "elements": {
                "200": {"element_id":200,"release_id":1,"type":"table","name":"B","level":"0"},
                "100": {"element_id":100,"release_id":1,"type":"table","name":"A","level":"0"}
            }
        }"#;
        let table: ReleaseTable = serde_json::from_str(body).unwrap();
        let ids: Vec<u32> = table.roots.iter().map(|e| e.element_id.get()).collect();
        assert_eq!(ids, [100, 200]);
    }
}
