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
        .stdout(predicate::str::contains("release"));
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
}
