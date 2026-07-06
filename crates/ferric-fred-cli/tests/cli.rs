//! Integration tests for the `fred` binary: run the compiled CLI as a
//! subprocess and assert on its exit status and output.
//!
//! The non-ignored tests are fully offline — they exercise argument parsing,
//! validation, and the missing-key error path, none of which touch the network
//! (they fail during `clap` parsing or before any request is built). The
//! `#[ignore]`d tests hit the live FRED API and need `FRED_API_KEY`.

use assert_cmd::Command;
use predicates::prelude::*;

/// A `Command` for the `fred` binary with `FRED_API_KEY` removed, so a key in
/// the ambient environment can't accidentally satisfy `Client::from_env`
/// during the offline tests.
fn fred() -> Command {
    let mut cmd = Command::cargo_bin("fred").expect("the `fred` binary builds");
    cmd.env_remove("FRED_API_KEY");
    cmd
}

#[test]
fn help_lists_every_subcommand() {
    fred()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("series"))
        .stdout(predicate::str::contains("observations"))
        .stdout(predicate::str::contains("chart"))
        .stdout(predicate::str::contains("category"))
        .stdout(predicate::str::contains("release"))
        .stdout(predicate::str::contains("source"))
        .stdout(predicate::str::contains("tags"))
        .stdout(predicate::str::contains("updates"))
        .stdout(predicate::str::contains("geofred"));
}

#[test]
fn geofred_regional_requires_all_fred_mandated_options() {
    // FRED requires region_type, date, units, frequency, and season for
    // regional/data; clap enforces them at parse time (ADR-0025).
    fred()
        .args(["geofred", "regional", "882"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("--region-type"))
        .stderr(predicate::str::contains("--season"));
}

#[test]
fn geofred_regional_rejects_an_invalid_region_type() {
    fred()
        .args([
            "geofred",
            "regional",
            "882",
            "--region-type",
            "planet",
            "--date",
            "2013-01-01",
            "--units",
            "Dollars",
            "--frequency",
            "annual",
            "--season",
            "nsa",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"))
        .stderr(predicate::str::contains("state"));
}

#[test]
fn geofred_series_data_date_and_start_date_conflict() {
    fred()
        .args([
            "geofred",
            "series-data",
            "SMU56000000500000001",
            "--date",
            "2013-01-01",
            "--start-date",
            "2010-01-01",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn updates_invalid_filter_lists_the_choices() {
    fred()
        .args(["updates", "--filter", "bogus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"))
        .stderr(predicate::str::contains("macro"));
}

#[test]
fn updates_start_time_requires_end_time() {
    // --start-time and --end-time are a required pair (clap `requires`); caught
    // at parse time, before any network call.
    fred()
        .args(["updates", "--start-time", "2024-03-01T14:30"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("--end-time"));
}

#[test]
fn updates_rejects_a_malformed_time() {
    fred()
        .args([
            "updates",
            "--start-time",
            "nope",
            "--end-time",
            "2024-03-02T00:00",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"))
        .stderr(predicate::str::contains("date-time"));
}

#[test]
fn series_views_are_mutually_exclusive() {
    // --tags / --categories / --release form a clap group; caught at parse time.
    fred()
        .args(["series", "GNPCA", "--tags", "--categories"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn source_releases_without_id_is_an_error() {
    // --releases requires a source id (clap `requires`); caught at parse time.
    fred()
        .args(["source", "--releases"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("<ID>"));
}

#[test]
fn release_series_without_id_is_an_error() {
    // --series requires a release id (clap `requires`); caught at parse time,
    // before any network call.
    fred()
        .args(["release", "--series"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("<ID>"));
}

#[test]
fn release_sources_without_id_is_an_error() {
    // --sources requires a release id (clap `requires`); caught at parse time.
    fred()
        .args(["release", "--sources"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("<ID>"));
}

#[test]
fn release_series_and_sources_conflict() {
    // --series and --sources are mutually exclusive (clap `conflicts_with`).
    fred()
        .args(["release", "53", "--series", "--sources"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn release_dates_conflicts_with_series() {
    // --dates is a distinct view; it can't combine with --series/--sources.
    fred()
        .args(["release", "53", "--series", "--dates"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn release_include_no_data_requires_dates() {
    // --include-no-data only makes sense with --dates (clap `requires`).
    fred()
        .args(["release", "53", "--include-no-data"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("--dates"));
}

#[test]
fn category_tag_views_are_mutually_exclusive() {
    // --tags, --series, and --related-tags share a clap view group.
    fred()
        .args(["category", "125", "--tags", "--series"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn category_related_conflicts_with_series() {
    // --related (related categories) is a distinct view from --series; the two
    // must not combine (and it is distinct from --related-tags).
    fred()
        .args(["category", "125", "--related", "--series"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn release_tags_requires_id() {
    // --tags scopes to a release, so it needs an id (clap `requires`).
    fred()
        .args(["release", "--tags"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("<ID>"));
}

#[test]
fn release_tags_conflicts_with_dates() {
    // --tags is a distinct view from --series/--sources/--dates.
    fred()
        .args(["release", "53", "--tags", "--dates"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn release_tables_requires_id() {
    // --tables scopes to a release, so it needs an id (clap `requires`).
    fred()
        .args(["release", "--tables"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("<ID>"));
}

#[test]
fn release_element_requires_tables() {
    // --element only makes sense with --tables (clap `requires`).
    fred()
        .args(["release", "10", "--element", "34483"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required arguments"))
        .stderr(predicate::str::contains("--tables"));
}

#[test]
fn release_tables_conflicts_with_series() {
    // --tables is a distinct view from the other release views.
    fred()
        .args(["release", "10", "--tables", "--series"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn search_tags_and_related_tags_conflict() {
    // The two tag views on `search` are mutually exclusive.
    fred()
        .args(["search", "gdp", "--tags", "--related-tags", "monthly"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("cannot be used with"));
}

#[test]
fn version_is_reported() {
    fred()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("fred"));
}

#[test]
fn no_subcommand_is_a_usage_error() {
    fred()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn unknown_subcommand_is_rejected() {
    fred()
        .arg("teleport")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn search_requires_text() {
    fred()
        .arg("search")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn invalid_date_is_rejected_before_any_request() {
    fred()
        .args(["observations", "GNPCA", "--start", "not-a-date"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn invalid_units_value_lists_the_choices() {
    fred()
        .args(["observations", "GNPCA", "--units", "bogus"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"))
        .stderr(predicate::str::contains("pch"));
}

#[test]
fn missing_api_key_is_a_friendly_error() {
    fred()
        .args(["series", "GNPCA"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("FRED_API_KEY"));
}

#[test]
fn json_is_a_global_flag() {
    fred()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--json"));
}

#[test]
fn all_is_a_global_flag() {
    fred()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("--all"));
}

// --- Live tests: hit the real FRED API and require FRED_API_KEY. Run with
// `--run-ignored all` (nextest) or `--ignored` inside the direnv/infisical
// shell. These build a fresh Command so the inherited FRED_API_KEY survives. ---

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn search_succeeds_against_live_fred() {
    Command::cargo_bin("fred")
        .unwrap()
        .args(["search", "unemployment rate", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("match(es)"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn search_all_pages_past_a_single_page() {
    // "gdp" matches far more than FRED's 1000-per-page cap, so `--all --limit
    // 1500` must page twice and return exactly 1500 — proving `--all` walks
    // past a single page and that `--limit` is a ceiling on the total.
    Command::cargo_bin("fred")
        .unwrap()
        .args(["search", "gdp", "--all", "--limit", "1500"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1500 match(es)"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn observations_succeeds_against_live_fred() {
    Command::cargo_bin("fred")
        .unwrap()
        .args(["observations", "GNPCA", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("observation(s)"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn json_output_emits_json() {
    Command::cargo_bin("fred")
        .unwrap()
        .args(["series", "GNPCA", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{"))
        .stdout(predicate::str::contains("\"id\": \"GNPCA\""));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn category_browse_lists_children() {
    // The root (id 0) always has child categories.
    Command::cargo_bin("fred")
        .unwrap()
        .args(["category", "0"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Categories"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn category_series_lists_series() {
    Command::cargo_bin("fred")
        .unwrap()
        .args(["category", "125", "--series", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("series in category 125"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn category_related_lists_related() {
    // 32073 (a regional category) has related categories (the states).
    Command::cargo_bin("fred")
        .unwrap()
        .args(["category", "32073", "--related"])
        .assert()
        .success()
        .stdout(predicate::str::contains("categories related to 32073"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn release_list_and_single_and_series() {
    let fred = || Command::cargo_bin("fred").unwrap();

    fred()
        .args(["release", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("releases:"));

    fred()
        .args(["release", "53"])
        .assert()
        .success()
        .stdout(predicate::str::contains("press release:"));

    fred()
        .args(["release", "53", "--series", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("series in release 53"));

    fred()
        .args(["release", "53", "--sources"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sources for release 53"));

    // The calendar across all releases, and one release's own dates.
    fred()
        .args(["release", "--dates", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("release dates:"));

    fred()
        .args(["release", "53", "--dates", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("release dates for release 53"));

    // The table tree of release 10 (CPI), scoped to a subtree by element id.
    fred()
        .args(["release", "10", "--tables", "--element", "34483"])
        .assert()
        .success()
        .stdout(predicate::str::contains("release 10 table —"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn source_list_and_single_and_releases() {
    let fred = || Command::cargo_bin("fred").unwrap();

    fred()
        .args(["source", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("sources:"));

    fred()
        .args(["source", "18"])
        .assert()
        .success()
        .stdout(predicate::str::contains("18:"));

    fred()
        .args(["source", "18", "--releases", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("releases from source 18"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn updates_lists_recently_updated_series() {
    Command::cargo_bin("fred")
        .unwrap()
        .args(["updates", "--filter", "macro", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("series updated recently"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn tags_browse_search_and_series() {
    let fred = || Command::cargo_bin("fred").unwrap();

    fred()
        .args(["tags", "--search-text", "gdp", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tags:"));

    fred()
        .args(["tags", "gdp", "quarterly", "--limit", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tagged gdp, quarterly"));

    fred()
        .args(["tags", "gdp", "--related", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tags related to gdp"));
}

#[test]
#[ignore = "needs FRED_API_KEY for client init, though it fails before any request"]
fn tags_related_without_names_is_an_error() {
    // The --related guard fires after the client is built, so a key must be set.
    Command::cargo_bin("fred")
        .unwrap()
        .args(["tags", "--related"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("needs one or more tag names"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn series_tags_lists_tags() {
    Command::cargo_bin("fred")
        .unwrap()
        .args(["series", "GNPCA", "--tags"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tags for GNPCA"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn series_categories_and_release_views() {
    let fred = || Command::cargo_bin("fred").unwrap();

    fred()
        .args(["series", "GNPCA", "--categories"])
        .assert()
        .success()
        .stdout(predicate::str::contains("categories for GNPCA"));

    fred()
        .args(["series", "GNPCA", "--release"])
        .assert()
        .success()
        .stdout(predicate::str::contains("release for GNPCA"));

    fred()
        .args(["series", "GNPCA", "--vintages"])
        .assert()
        .success()
        .stdout(predicate::str::contains("vintage dates for GNPCA"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn scoped_tag_facet_views() {
    let fred = || Command::cargo_bin("fred").unwrap();

    fred()
        .args(["category", "125", "--tags", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tags in category 125"));

    fred()
        .args(["category", "125", "--related-tags", "trade", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "tags related to trade in category 125",
        ));

    fred()
        .args(["release", "53", "--tags", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tags in release 53"));

    fred()
        .args(["release", "53", "--related-tags", "gdp", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "tags related to gdp in release 53",
        ));

    fred()
        .args(["search", "unemployment", "--tags", "--limit", "3"])
        .assert()
        .success()
        .stdout(predicate::str::contains("tags for series matching"));

    fred()
        .args([
            "search",
            "unemployment",
            "--related-tags",
            "monthly",
            "--limit",
            "3",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "tags related to monthly among series matching",
        ));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn geofred_regional_lists_regions() {
    // Series group 882 = Per Capita Personal Income; a state cross-section.
    Command::cargo_bin("fred")
        .unwrap()
        .args([
            "geofred",
            "regional",
            "882",
            "--region-type",
            "state",
            "--date",
            "2013-01-01",
            "--units",
            "Dollars",
            "--frequency",
            "annual",
            "--season",
            "nsa",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("2013-01-01:"))
        .stdout(predicate::str::contains("Alabama"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn geofred_group_shows_metadata() {
    Command::cargo_bin("fred")
        .unwrap()
        .args(["geofred", "group", "SMU56000000500000001"])
        .assert()
        .success()
        .stdout(predicate::str::contains("region type: state"));
}

#[test]
#[ignore = "hits the live FRED API; requires FRED_API_KEY"]
fn geofred_shapes_summarizes_and_json_emits_geojson() {
    Command::cargo_bin("fred")
        .unwrap()
        .args(["geofred", "shapes", "--shape", "bea"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feature(s)"));

    Command::cargo_bin("fred")
        .unwrap()
        .args(["geofred", "shapes", "--shape", "bea", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"FeatureCollection\""));
}
