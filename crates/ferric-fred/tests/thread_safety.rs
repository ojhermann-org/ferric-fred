//! Locks the public types' auto-trait profile (ADR-0029, lesson **L5**; canonical
//! owner `time-value` ADR-0046): the owned public types are `Send + Sync +
//! 'static`, and the borrowing request builders are `Send + Sync`.
//!
//! Everything here holds trivially today — `Client` is a `reqwest::Client` (an
//! `Arc` pool behind the scenes) plus `String`s, the domain/return types are
//! plain data, and the request builders just borrow the client. So the test is
//! green immediately. Its job is to *lock* the profile: it stops compiling the
//! moment a field regresses it — an `Rc`, `Cell`, or `RefCell` slipped into a
//! public type silently dropping `Send`/`Sync` — turning an otherwise invisible
//! semver break into a build error. The lock is zero-cost and zero-dependency
//! (no `static_assertions`): the discipline is ADRs + tests, not tooling.
//!
//! For an async FRED client this profile *enables* use rather than constraining
//! it: a `Send + Sync + 'static` `Client` crosses `.await` points, is held across
//! `tokio::spawn`, and is shared through an `Arc` — it only constrains us (no
//! non-thread-safe fields on a public type).

use ferric_fred::*;

fn assert_send_sync_static<T: Send + Sync + 'static>() {}
fn assert_send_sync<T: Send + Sync>() {}

#[test]
fn owned_public_types_are_send_sync_static() {
    // The client handle.
    assert_send_sync_static::<Client>();

    // Error surface.
    assert_send_sync_static::<Error>();

    // Id newtypes.
    assert_send_sync_static::<CategoryId>();
    assert_send_sync_static::<ReleaseElementId>();
    assert_send_sync_static::<ReleaseId>();
    assert_send_sync_static::<SeriesGroupId>();
    assert_send_sync_static::<SeriesId>();
    assert_send_sync_static::<SourceId>();

    // Closed/open vocabularies (`#[non_exhaustive]` enums).
    assert_send_sync_static::<AggregationMethod>();
    assert_send_sync_static::<Frequency>();
    assert_send_sync_static::<OrderBy>();
    assert_send_sync_static::<RegionType>();
    assert_send_sync_static::<SearchType>();
    assert_send_sync_static::<SeasonalAdjustment>();
    assert_send_sync_static::<ShapeType>();
    assert_send_sync_static::<SortOrder>();
    assert_send_sync_static::<Units>();
    assert_send_sync_static::<UpdatesFilter>();

    // Domain / return types.
    assert_send_sync_static::<Category>();
    assert_send_sync_static::<Observation>();
    assert_send_sync_static::<RegionalData>();
    assert_send_sync_static::<RegionalDataMeta>();
    assert_send_sync_static::<RegionalDataPoint>();
    assert_send_sync_static::<Release>();
    assert_send_sync_static::<ReleasesResults>();
    assert_send_sync_static::<ReleaseDate>();
    assert_send_sync_static::<ReleaseDatesResults>();
    assert_send_sync_static::<ReleaseTable>();
    assert_send_sync_static::<ReleaseTableElement>();
    assert_send_sync_static::<Series>();
    assert_send_sync_static::<SeriesGroup>();
    assert_send_sync_static::<SeriesSearchResults>();
    assert_send_sync_static::<Source>();
    assert_send_sync_static::<SourcesResults>();
    assert_send_sync_static::<Tag>();
    assert_send_sync_static::<TagsResults>();
    assert_send_sync_static::<VintageDates>();

    // GeoFRED shape-file types.
    assert_send_sync_static::<Feature>();
    assert_send_sync_static::<Geometry>();
    assert_send_sync_static::<ShapeFile>();
}

#[test]
fn borrowing_request_builders_are_send_sync() {
    // Each builder borrows the client with some lifetime `'a` (introduced by the
    // reference parameter); they are `Send + Sync` for any `'a` — their `Sync`
    // rides on the borrowed `Client: Sync` — independent of `'static`.
    fn check<'a>(_witness: &'a ()) {
        assert_send_sync::<ObservationsRequest<'a>>();
        assert_send_sync::<ReleaseDatesRequest<'a>>();
        assert_send_sync::<ReleaseTablesRequest<'a>>();
        assert_send_sync::<ReleasesRequest<'a>>();
        assert_send_sync::<SeriesDataRequest<'a>>();
        assert_send_sync::<SeriesListRequest<'a>>();
        assert_send_sync::<SeriesSearchRequest<'a>>();
        assert_send_sync::<SeriesUpdatesRequest<'a>>();
        assert_send_sync::<SourcesRequest<'a>>();
        assert_send_sync::<TagsRequest<'a>>();
        assert_send_sync::<VintageDatesRequest<'a>>();
    }
    check(&());
}
