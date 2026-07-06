# ferric-fred-cli

[![Crates.io](https://img.shields.io/crates/v/ferric-fred-cli.svg)](https://crates.io/crates/ferric-fred-cli)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/ferric-fred-cli.svg)](#license)

`fred` — a command-line interface to [FRED](https://fred.stlouisfed.org/)
(Federal Reserve Economic Data), built on the
[`ferric-fred`](https://crates.io/crates/ferric-fred) client. Search and inspect
series, print or **chart** observations in the terminal, browse categories,
releases, sources, and tags, and pull **GeoFRED** regional data and map shapes —
as text or JSON.

## Install

```sh
cargo install ferric-fred-cli   # installs the `fred` binary
```

Set a free [FRED API key](https://fredaccount.stlouisfed.org/apikeys) in the
`FRED_API_KEY` environment variable.

## Usage

```sh
fred search "unemployment rate" --order-by popularity --limit 3  # find series by text
fred series GNPCA                                                # a series' metadata
fred observations GDP --units pch --sort desc --limit 4          # transformed observations
fred chart GNPCA --start 1950-01-01                              # interactive terminal chart
fred category 125 --series --limit 5                             # series in a category
fred release 53 --series --limit 5                               # series in a release
fred source 18 --releases --limit 5                              # releases produced by a source
fred updates --filter macro --limit 10                           # recently updated series
fred tags gdp quarterly --limit 5                                # series carrying all these tags
fred geofred group SMU56000000500000001                          # GeoFRED series-group metadata
fred geofred regional 882 --region-type state --date 2013-01-01 \
  --units Dollars --frequency annual --season nsa                # a region cross-section
fred geofred shapes --shape bea --json                           # region boundary GeoJSON
fred source --all                                                # page a list to exhaustion
fred series GNPCA --json | jq .frequency                         # JSON output for scripting
```

- **`--json`** on any data command emits its domain type as JSON (`chart` ignores it).
- **`--all`** on any list view pages it to exhaustion instead of returning just
  the first page; `--limit` then caps the total (mind FRED's rate limits on large
  lists).
- **`fred chart`** opens an interactive [ratatui](https://ratatui.rs/) line chart;
  press `q`, `Esc`, or `Ctrl-C` to quit.

Run `fred <command> --help` for every flag — `--units`, `--order-by`, and friends
accept FRED's value sets.

## Documentation & source

Full command reference, examples, and design ADRs live in the repository:
[github.com/ojhermann-org/ferric-fred](https://github.com/ojhermann-org/ferric-fred).

## License

Dual-licensed under **MIT OR Apache-2.0**, at your option. FRED data itself is
subject to the St. Louis Fed's terms of use; you supply your own API key.
