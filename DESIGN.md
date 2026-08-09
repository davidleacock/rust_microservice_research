# DESIGN.md — TaskFlow

Living design doc. Updated after each phase lands — not speculatively ahead of it.
See `CLAUDE.md` for how Claude and Dave work together on this project.

## Overview & Goal

TaskFlow is a small task/work-item tracking service (Jira/Linear-shaped backend),
built incrementally as a portfolio-quality, idiomatic-Rust microservice. Dave writes
all the code; Claude defines each phase's requirements and reviews against them.
The goal is credibility in a Rust job interview — code that reads like something a
senior Rust developer wrote, not "Scala with semicolons."

## Architecture Principles

- **Domain core stays I/O-free.** Business rules and validation live in plain Rust
  types with no knowledge of HTTP, databases, or any transport.
- **Ports appear only when an adapter needs them.** No repository trait, no
  `dyn`-based abstraction, until there's a second implementation (e.g. a real
  persistence layer) that actually requires the seam. Introducing a trait "for
  future flexibility" before that point is a phase failure, not a nice-to-have.
- **Newtype everything that isn't structurally interchangeable.** If two things are
  both `String` or both `u64` but mean different things, they get distinct types.
- **State machines are enums**, not status strings or boolean flags.

## Phase Roadmap

| Phase | Focus | Status |
|---|---|---|
| 1 | Domain model (pure, no I/O) | In progress |
| 2 | HTTP API (Axum) over the domain | Not started |
| 3 | Persistence (repository port + real adapter) | Not started |
| 4+ | TBD — likely candidates: auth, observability, gRPC (tonic) alongside REST | Not started |

Phases 3+ are intentionally left thin here; each gets fully specified in this doc
only once the prior phase has passed review, so scope reflects what was actually
learned, not a guess made upfront.

## Phase 1 — Domain Model

**Scope:** pure domain types and business rules for tasks and projects. No traits,
no persistence, no HTTP, no async.

- `TaskId`, `ProjectId` — newtypes wrapping `Uuid`
- `Status` — enum: `Todo`, `InProgress`, `Done`, `Cancelled`, with a `transition_to`
  method that enforces valid transitions (invalid transitions return `Err`, not a
  panic)
- `Priority` — enum (e.g. `Low`, `Medium`, `High`)
- `Task` — struct with validated construction (e.g. rejects an empty title) returning
  `Result<Task, TaskError>`
- `TaskError` — via `thiserror`

**Explicitly out of scope for Phase 1:** persistence, any trait/port, HTTP, async,
`Project`-level aggregation logic beyond holding an ID reference.

**Acceptance criteria (checked at review, before Phase 2 is defined):**

- [ ] `cargo build` succeeds
- [ ] `cargo test` passes, with tests covering valid transitions, at least one
      rejected transition, and at least one rejected `Task` construction
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo fmt --check` clean
- [ ] No `unwrap()`/`expect()` outside test code
- [ ] No trait/port introduced ahead of need
- [ ] IDs and status are newtypes/enums, not bare `String`/primitives

## Decision Log

- 2026-08-09 — Chose Axum (HTTP) before tonic (gRPC) for the API phase: closer to
  what most Rust job interviews and day-one work expect; gRPC added once the domain
  and REST layer are solid.
- 2026-08-09 — Deferred the repository trait to Phase 3 rather than defining it in
  Phase 1: avoids designing a port before an adapter exists to justify its shape.
