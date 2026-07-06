# Ideas & explorations

A lightweight backlog of *"should we do X?"* ideas worth remembering but **not**
worth investing in while the focus is on ferric-fred's core mission. This is not a
commitment list — entries here are candidates, not planned work.

**Pipeline.** An idea lives here until we decide to pursue it. When we do, it
graduates into an [ADR](adr/README.md) (the decision + design) and then
implementation; the entry below is updated to *promoted to ADR-NNNN*. Ideas we
rule out are marked *dropped* with a one-line reason, not deleted — the "why not"
is worth keeping.

**Scope guardrail.** ferric-fred is a **faithful mirror of FRED's API surface** —
it fetches and forwards what FRED provides, and nothing derived (calculations,
map rendering, coordinate transforms, unit math). Ideas that add derived behaviour
belong in a *separate* library, not this one; recording them here keeps them from
scope-creeping into the client.

**Adding one:** append a row — *idea · why · status* — in a normal PR.

| Idea | Why it's interesting | Why it's parked | Status |
|---|---|---|---|
| A separate library for FRED-derived calculations | Transforms, unit math, map rendering over FRED/GeoFRED data — real value, but *computation*, not API access | Would violate ferric-fred's "faithful mirror" guardrail; belongs in its own crate so the client stays lean and true to FRED | Open |
| Expose GeoFRED `shapes/file` as an MCP tool | Full tool parity across library/CLI/MCP; a caller wanting raw GeoJSON could get it over MCP | A tens-of-KB `MultiPolygon` blob in a display projection is poor ergonomics for an LLM tool caller; deferred in [ADR-0025](adr/0025-geofred-maps-api.md) | Open |
