use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Deserializer, Serialize};

use crate::{ReleaseElementId, ReleaseId, SeriesId};

/// A release's table tree, from the `fred/release/tables` endpoint — the layout
/// a release uses to present its series (sections and tables, with series rows
/// nested beneath them).
///
/// FRED returns the top-level `elements` as a JSON object keyed by element id,
/// each value carrying its subtree inline via
/// [`children`](ReleaseTableElement::children). The object is *flattened* — for a
/// subtree request it also keys every descendant, not just the roots — so we keep
/// only the true roots (those whose parent isn't itself in the object) as the
/// ordered [`roots`](ReleaseTable::roots) vector, leaving deeper nodes reachable
/// solely through `children` (see `roots_from_map`). `name` and `element_id`
/// are present only when a subtree was requested (see
/// [`ReleaseTablesRequest::element`](crate::ReleaseTablesRequest::element)); for a
/// whole-release request they are absent.
// `Eq` is intentionally omitted: `ReleaseTableElement` carries an `f64`
// observation value (which is only `PartialEq`), so the tree is `PartialEq` only,
// mirroring [`Observation`](crate::Observation).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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
    /// On the wire FRED names this `elements` (a flattened object keyed by id);
    /// we read that, keep only the tree's true roots, and re-serialize as a
    /// `roots` array (see `roots_from_map`).
    #[serde(
        rename(serialize = "roots", deserialize = "elements"),
        deserialize_with = "roots_from_map"
    )]
    pub roots: Vec<ReleaseTableElement>,
}

/// A node in a release's table tree: a section, a table, or a series row. Nodes
/// nest via [`children`](ReleaseTableElement::children) to arbitrary depth.
// `Eq` is intentionally omitted — `observation_value: Option<f64>` is `PartialEq`
// only; see the note on [`ReleaseTable`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
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

    /// The element's observation value at the request's `observation_date` (or
    /// FRED's latest), present only when the request set
    /// [`include_observation_values`](crate::ReleaseTablesRequest::include_observation_values).
    /// `None` for a structural (non-`series`) element, when values weren't
    /// requested, or when FRED reports the value as missing (`"."`) — mirroring
    /// [`Observation`](crate::Observation)'s value handling.
    #[serde(default, deserialize_with = "deserialize_optional_value")]
    pub observation_value: Option<f64>,

    /// FRED's formatted label for the [`observation_value`](ReleaseTableElement::observation_value)
    /// date, e.g. `"Jun 2023"` or `"2023"` — a display string keyed to the
    /// series' frequency, **not** an ISO date (unlike the request's
    /// `observation_date`). `None` when values weren't requested or the element
    /// carries no series.
    #[serde(default)]
    pub observation_date: Option<String>,

    /// The child elements nested beneath this one (empty for a leaf).
    #[serde(default)]
    pub children: Vec<ReleaseTableElement>,
}

/// Deserialize FRED's `elements` object (keyed by element id) into the ordered
/// vector of *root* elements of the returned tree.
///
/// FRED flattens the object: for a subtree request (`element_id`) it keys
/// **every descendant** of the requested element, and each value *also* carries
/// its own subtree inline via [`children`](ReleaseTableElement::children). Taking
/// every value as a root would therefore surface each non-root node twice — once
/// here and once under its parent's `children` — so a consumer walking the tree
/// double-counts. A value is a true root of the returned tree only when its
/// `parent_id` is absent from the object's keys (the requested element itself is
/// not in the object, so its direct children qualify; a whole-release request's
/// top-level sections carry a null `parent_id` and qualify too). Every other node
/// stays reachable solely through its parent's `children`. Ordering is by element
/// id, so the result is deterministic regardless of the object's key order.
fn roots_from_map<'de, D>(deserializer: D) -> Result<Vec<ReleaseTableElement>, D::Error>
where
    D: Deserializer<'de>,
{
    // Keys are stringified ids; each value already carries its own element_id.
    let map: BTreeMap<String, ReleaseTableElement> = BTreeMap::deserialize(deserializer)?;
    let ids: HashSet<ReleaseElementId> = map.values().map(|element| element.element_id).collect();
    let mut roots: Vec<ReleaseTableElement> = map
        .into_values()
        .filter(|element| {
            element
                .parent_id
                .is_none_or(|parent| !ids.contains(&parent))
        })
        .collect();
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

/// Deserialize a release-table element's `observation_value`: `"."` and the
/// empty string → `None`, otherwise parse the string as `f64`. Unlike the
/// `observations` endpoint's raw values, `release/tables` returns
/// **display-formatted** strings in US format — comma thousands-separators with
/// a `.` decimal point (a GDP aggregate arrives as `"27,000.0"`, not `"27000.0"`) —
/// so the commas are stripped before parsing; the `.` decimal point is left
/// intact. Mirrors [`Observation`](crate::Observation)'s value handling; paired
/// with `#[serde(default)]`, so an absent field — values not requested, or a
/// structural element — also yields `None`. A present, non-`"."`, non-empty
/// value that still fails to parse is an error, not a silent `None`.
fn deserialize_optional_value<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    if raw == "." || raw.is_empty() {
        return Ok(None);
    }
    raw.replace(',', "")
        .parse::<f64>()
        .map(Some)
        .map_err(serde::de::Error::custom)
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

    /// A subtree request: FRED *flattens* `elements`, keying **every descendant**
    /// of the requested element (here 12886) while *also* nesting each node's
    /// subtree under its parent's `children`. Only the requested element's direct
    /// children (12887, 12890) are true roots; 12888/12889 must appear solely
    /// under 12887, never promoted into `roots`. Regression test for the
    /// descendant-duplication bug (#55).
    #[test]
    fn subtree_request_keeps_only_true_roots_without_duplication() {
        let body = r#"{
            "name": "Personal consumption expenditures",
            "element_id": 12886,
            "release_id": "53",
            "elements": {
                "12887": {
                    "element_id": 12887, "release_id": 53, "parent_id": 12886,
                    "series_id": null, "type": "section", "name": "Goods",
                    "line": null, "level": "1",
                    "children": [
                        {
                            "element_id": 12888, "release_id": 53, "parent_id": 12887,
                            "series_id": "DDURRL1A225NBEA", "type": "series",
                            "name": "Durable goods", "line": null, "level": "2",
                            "children": []
                        },
                        {
                            "element_id": 12889, "release_id": 53, "parent_id": 12887,
                            "series_id": "DNDGRL1A225NBEA", "type": "series",
                            "name": "Nondurable goods", "line": null, "level": "2",
                            "children": []
                        }
                    ]
                },
                "12888": {
                    "element_id": 12888, "release_id": 53, "parent_id": 12887,
                    "series_id": "DDURRL1A225NBEA", "type": "series",
                    "name": "Durable goods", "line": null, "level": "2", "children": []
                },
                "12889": {
                    "element_id": 12889, "release_id": 53, "parent_id": 12887,
                    "series_id": "DNDGRL1A225NBEA", "type": "series",
                    "name": "Nondurable goods", "line": null, "level": "2", "children": []
                },
                "12890": {
                    "element_id": 12890, "release_id": 53, "parent_id": 12886,
                    "series_id": null, "type": "section", "name": "Services",
                    "line": null, "level": "1", "children": []
                }
            }
        }"#;
        let table: ReleaseTable = serde_json::from_str(body).unwrap();
        assert_eq!(table.element_id, Some(ReleaseElementId::new(12886)));

        // Only the requested element's direct children are roots.
        let root_ids: Vec<u32> = table.roots.iter().map(|e| e.element_id.get()).collect();
        assert_eq!(root_ids, vec![12887, 12890]);

        // 12888 / 12889 are reachable only under 12887, not promoted to roots.
        let goods = &table.roots[0];
        assert_eq!(goods.element_id, ReleaseElementId::new(12887));
        let child_ids: Vec<u32> = goods.children.iter().map(|e| e.element_id.get()).collect();
        assert_eq!(child_ids, vec![12888, 12889]);

        // No element id appears more than once across the whole tree.
        fn collect(node: &ReleaseTableElement, out: &mut Vec<u32>) {
            out.push(node.element_id.get());
            for child in &node.children {
                collect(child, out);
            }
        }
        let mut all = Vec::new();
        for root in &table.roots {
            collect(root, &mut all);
        }
        let mut deduped = all.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(all.len(), deduped.len(), "no element should appear twice");
    }

    #[test]
    fn observation_values_deserialize_when_present() {
        // Mirrors the live `include_observation_values=true` shape: series rows
        // carry `observation_value` (a stringly-typed number, or "." for
        // missing) and a frequency-formatted `observation_date`; structural
        // elements carry neither.
        let body = r#"{
            "release_id": "10",
            "elements": {
                "36714": {
                    "element_id": 36714, "release_id": 10, "type": "table",
                    "name": "Monthly, Seasonally Adjusted", "level": "0",
                    "children": [
                        {
                            "element_id": 36715, "release_id": 10, "parent_id": 36714,
                            "series_id": "CUSR0000SA0L5", "type": "series",
                            "name": "All items less medical care", "level": "1",
                            "observation_value": "292.260", "observation_date": "Jun 2023",
                            "children": []
                        },
                        {
                            "element_id": 36716, "release_id": 10, "parent_id": 36714,
                            "series_id": "CPILEGSL", "type": "series", "name": "Missing",
                            "level": "1",
                            "observation_value": ".", "observation_date": "Jun 2023",
                            "children": []
                        }
                    ]
                }
            }
        }"#;
        let table: ReleaseTable = serde_json::from_str(body).unwrap();
        let table_elem = &table.roots[0];
        // Structural element: no value, no date.
        assert_eq!(table_elem.observation_value, None);
        assert_eq!(table_elem.observation_date, None);

        let with_value = &table_elem.children[0];
        assert_eq!(with_value.observation_value, Some(292.260));
        assert_eq!(with_value.observation_date.as_deref(), Some("Jun 2023"));

        // FRED's "." sentinel maps to a missing value, not a parse error.
        let missing = &table_elem.children[1];
        assert_eq!(missing.observation_value, None);
        assert_eq!(missing.observation_date.as_deref(), Some("Jun 2023"));
    }

    #[test]
    fn comma_formatted_observation_values_deserialize() {
        // `release/tables` returns display-formatted values: US formatting with
        // comma thousands-separators (GDP dollar aggregates like "27,000.0").
        // Regression test for #77 — these must parse, not blow up the whole tree
        // with `invalid float literal`. Also pins empty-string → None alongside
        // the "." sentinel, and that a genuinely non-numeric value still errors.
        let body = |value: &str| {
            format!(
                r#"{{
                    "release_id": "53",
                    "elements": {{
                        "12998": {{
                            "element_id": 12998, "release_id": 53, "type": "series",
                            "series_id": "GDP", "name": "Gross domestic product",
                            "level": "0",
                            "observation_value": "{value}", "observation_date": "2023",
                            "children": []
                        }}
                    }}
                }}"#
            )
        };

        let parse = |value: &str| -> Result<Option<f64>, _> {
            serde_json::from_str::<ReleaseTable>(&body(value))
                .map(|table| table.roots[0].observation_value)
        };

        // Comma thousands-separator, decimal point preserved.
        assert_eq!(parse("27,000.0").unwrap(), Some(27000.0));
        // Multiple commas (millions) round-trip too.
        assert_eq!(parse("1,234,567.89").unwrap(), Some(1234567.89));
        // No separator (small value) still parses.
        assert_eq!(parse("332.568").unwrap(), Some(332.568));
        // Missing-value sentinels.
        assert_eq!(parse(".").unwrap(), None);
        assert_eq!(parse("").unwrap(), None);
        // A present, non-numeric value is still a hard error, not a silent None.
        assert!(parse("N/A").is_err());
    }

    #[test]
    fn observation_values_absent_when_not_requested() {
        // The base (structure-only) shape has no value/date fields at all.
        let table: ReleaseTable = serde_json::from_str(TABLE_BODY).unwrap();
        let leaf = &table.roots[0].children[0].children[0];
        assert_eq!(leaf.observation_value, None);
        assert_eq!(leaf.observation_date, None);
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
