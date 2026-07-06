//! Live tests for the source endpoints; hit the real FRED API and require
//! `FRED_API_KEY`. Run with `--run-ignored all` (nextest) inside the dev shell.

use ferric_fred::{Client, Paginate, SourceId};

#[tokio::test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
async fn sources_list_and_single_and_releases() {
    let client = Client::from_env().expect("FRED_API_KEY set");

    // The full list of sources is non-empty.
    let results = client.sources().limit(5).send().await.expect("sources");
    assert!(results.count > 0);
    assert!(!results.sources.is_empty());

    // `send_all` walks every page: with no `.limit()` ceiling it returns exactly
    // `count` sources (FRED has well over one page of them).
    let all = client.sources().send_all().await.expect("all sources");
    assert_eq!(all.len(), results.count as usize);

    // `stream` yields the same items lazily; counting them agrees with `count`.
    use futures_util::TryStreamExt;
    let streamed: Vec<_> = client
        .sources()
        .stream()
        .try_collect()
        .await
        .expect("streamed sources");
    assert_eq!(streamed.len(), results.count as usize);

    // 18 = "U.S. Bureau of Economic Analysis", a stable source.
    let source = client.source(SourceId::new(18)).await.expect("source 18");
    assert_eq!(source.id, SourceId::new(18));
    assert!(!source.name.is_empty());

    // Its releases.
    let releases = client
        .source_releases(SourceId::new(18))
        .limit(1)
        .send()
        .await
        .expect("source 18 releases");
    assert!(releases.count > 0);
}
