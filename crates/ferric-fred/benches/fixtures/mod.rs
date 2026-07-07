//! Shared synthetic-fixture builders for the deserialization benches.
//!
//! Deterministic (no RNG, no clock) so results are comparable across machines
//! and across CI runs. The JSON shape mirrors FRED's `series/observations`
//! response envelope exactly — the same bytes `Client::execute_observations`
//! hands to `serde_json::from_slice`, so the benched work is the real parse
//! path (`deserialize_value` on the folded string value, plus two `NaiveDate`
//! parses per row).
//!
//! Lives in a `benches/` subdirectory (not a top-level `.rs`) so cargo does not
//! auto-discover it as its own bench target; each bench includes it with
//! `#[path = "fixtures/mod.rs"] mod fixtures;`.

use std::fmt::Write as _;

/// Build a `series/observations` response body carrying `rows` observations.
///
/// Roughly 1 value in 12 is FRED's missing sentinel `"."` (exercising the
/// `deserialize_value` `None` branch); the rest are decimals that parse to
/// `Some(f64)`. When `vintages > 1` the real-time window is widened per row to
/// mimic ALFRED archival vintages (distinct `realtime_start`/`realtime_end`
/// rather than the today/today of a latest query).
#[must_use]
pub fn observations_body(rows: usize, vintages: usize) -> Vec<u8> {
    let mut s = String::with_capacity(rows * 96 + 32);
    s.push_str("{\"observations\":[");
    for i in 0..rows {
        if i > 0 {
            s.push(',');
        }
        let (y, m, d) = walk_date(i);
        let (rs, re) = if vintages > 1 {
            // Widen the realtime window across `vintages` buckets so distinct
            // archival periods appear, as ALFRED vintage queries return.
            let (vy, vm, vd) = walk_date(i / vintages);
            (
                format!("{vy:04}-{vm:02}-{vd:02}"),
                format!("{:04}-{vm:02}-{vd:02}", vy + 1),
            )
        } else {
            ("2026-07-07".to_owned(), "2026-07-07".to_owned())
        };
        let value = if i % 12 == 0 {
            ".".to_owned()
        } else {
            format!("{}.{:03}", (i % 9000) + 100, i % 1000)
        };
        write!(
            s,
            "{{\"realtime_start\":\"{rs}\",\"realtime_end\":\"{re}\",\"date\":\"{y:04}-{m:02}-{d:02}\",\"value\":\"{value}\"}}"
        )
        .expect("writing to a String cannot fail");
    }
    s.push_str("]}");
    s.into_bytes()
}

/// A crude but always-valid calendar walk: step `n` positions from 1900-01-01
/// using a fixed 28-day month so every generated date is real (no Feb-30). We
/// exercise `NaiveDate`'s *parser*, not calendar arithmetic, so a dense valid
/// date space is all that matters.
fn walk_date(n: usize) -> (usize, usize, usize) {
    let day = n % 28 + 1;
    let month = (n / 28) % 12 + 1;
    let year = 1900 + n / (28 * 12);
    (year, month, day)
}
