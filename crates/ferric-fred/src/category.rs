use serde::{Deserialize, Serialize};

use crate::CategoryId;

/// A node in the FRED category tree (the `fred/category` and
/// `fred/category/children` endpoints).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category {
    /// The category's identifier.
    pub id: CategoryId,

    /// Human-readable name, e.g. `"Trade Balance"`.
    pub name: String,

    /// The parent category's id. For the root category this is
    /// [`CategoryId::ROOT`] (`0`), which FRED may also omit entirely.
    #[serde(default)]
    pub parent_id: CategoryId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_a_category() {
        let category: Category =
            serde_json::from_str(r#"{"id":125,"name":"Trade Balance","parent_id":13}"#).unwrap();
        assert_eq!(category.id, CategoryId::new(125));
        assert_eq!(category.name, "Trade Balance");
        assert_eq!(category.parent_id, CategoryId::new(13));
    }

    #[test]
    fn parent_id_defaults_to_root_when_absent() {
        let category: Category = serde_json::from_str(r#"{"id":0,"name":"Categories"}"#).unwrap();
        assert_eq!(category.parent_id, CategoryId::ROOT);
    }

    #[test]
    fn serializes_ids_as_bare_integers() {
        let category = Category {
            id: CategoryId::new(13),
            name: "U.S. Trade & International Transactions".to_owned(),
            parent_id: CategoryId::ROOT,
        };
        let value = serde_json::to_value(&category).unwrap();
        assert_eq!(value["id"], 13);
        assert_eq!(value["parent_id"], 0);
    }
}
