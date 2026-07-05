//! Live tests for the category endpoints; hit the real FRED API and require
//! `FRED_API_KEY`. Run with `--run-ignored all` (nextest) inside the dev shell.

use ferric_fred::{CategoryId, Client};

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn category_tree_and_series() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    // The root has children (the top-level categories).
    let children = client
        .category_children(CategoryId::ROOT)
        .await
        .expect("root children");
    assert!(!children.is_empty());

    // 125 = "Trade Balance", a stable leaf-ish category.
    let category = client
        .category(CategoryId::new(125))
        .await
        .expect("category 125");
    assert_eq!(category.id, CategoryId::new(125));

    let series = client
        .category_series(CategoryId::new(125))
        .limit(1)
        .send()
        .await
        .expect("category 125 series");
    assert!(series.count > 0);

    // Related categories: the endpoint resolves and deserializes (the list may
    // legitimately be empty for a given category, so we only assert well-formed
    // entries, not a count).
    let related = client
        .category_related(CategoryId::new(125))
        .await
        .expect("category 125 related");
    assert!(related.iter().all(|category| !category.name.is_empty()));
}
