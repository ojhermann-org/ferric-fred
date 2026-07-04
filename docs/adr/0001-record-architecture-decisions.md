# ADR-0001: Record architecture decisions

- **Status:** Accepted
- **Date:** 2026-07-04
- **Deciders:** Project owner

## Context

`ferric-fred` is a multi-crate project — a typed FRED library plus a CLI and an
MCP server that depend on it — with a cross-cutting release/versioning story.
Decisions made in the library ripple into the consumers and into CI. We want a
durable, reviewable record of *why* things are the way they are, so that future
changes (ours or contributors') are adaptations of known reasoning rather than
re-litigation from scratch.

## Decision

We will record significant architecture and process decisions as Architecture
Decision Records (ADRs) in `docs/adr/`, using a lightweight
[MADR](https://adr.github.io/madr/)-style Markdown template
([`0000-adr-template.md`](0000-adr-template.md)).

- ADRs are numbered sequentially (`NNNN-kebab-title.md`).
- Each has a **Status** (Proposed → Accepted, later possibly Deprecated or
  Superseded by another ADR).
- ADRs are immutable once Accepted: we don't rewrite history; we supersede an
  ADR with a new one and update the old one's status/link.
- The [index](README.md) lists all ADRs and their status.

ADRs are a source of truth that *guides* the build. They are expected to evolve
— superseding an ADR as we learn is the intended workflow, not a failure of the
original.

## Consequences

- Onboarding and future changes start from recorded reasoning.
- A small ongoing cost: significant decisions require writing them down.
- We need discipline to supersede rather than silently edit accepted ADRs.

## Alternatives considered

- **No formal record** — decisions live in commit messages and memory. Cheapest
  up front, but the "why" erodes quickly, exactly the failure mode we want to
  avoid on a project with linked crates and a release story.
- **A single design doc** — one growing document. Harder to see what changed and
  when, and tends to blur superseded decisions with current ones.
