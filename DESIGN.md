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
| 1 | Domain model (pure, no I/O) | Done |
| 2 | HTTP API (Axum) over the domain | Done |
| 3 | Persistence (repository port + real adapter) | In progress |
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

- [x] `cargo build` succeeds
- [x] `cargo test` passes, with tests covering valid transitions, at least one
      rejected transition, and at least one rejected `Task` construction
- [x] `cargo clippy --all-targets -- -D warnings` clean
- [x] `cargo fmt --check` clean
- [x] No `unwrap()`/`expect()` outside test code
- [x] No trait/port introduced ahead of need
- [x] IDs and status are newtypes/enums, not bare `String`/primitives

**Passed review 2026-08-22.** Final shape: `Task` exposes its fields only through
read-only accessors (`title() -> &str`, `task_id() -> TaskId`, `project_id() ->
Option<ProjectId>`) plus validated mutators (`set_priority`, `set_project_id`,
`set_status`) — no field is publicly writable directly, so `status` can only change
through `transition_to`'s validation. `TaskId`/`ProjectId` derive `Copy, Clone` so ID
values can be handed out by accessors without tying callers to `Task`'s borrow.

## Phase 2 — HTTP API (Axum)

**Scope:** an Axum HTTP layer over the Phase 1 domain — routes and handlers that
construct/read `Task`s through the accessor/mutator API already in place. No
persistence yet: an in-memory store (e.g. a `Mutex<HashMap<TaskId, Task>>` behind
`axum::extract::State`) is acceptable scaffolding here, precisely because it's a
placeholder to be replaced by the real repository port in Phase 3, not a designed
abstraction of its own.

**Explicitly out of scope for Phase 2:** a repository trait/port (still deferred to
Phase 3 — the in-memory map is a concrete stand-in, not a port), a real database,
auth, gRPC.

**Acceptance criteria:**

- [x] Endpoints for creating a task, reading a task by ID, updating priority/status,
      listing tasks
- [x] Domain errors (`TaskError`) map to sensible HTTP status codes, not a blanket 500
- [x] `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` clean
- [x] Handlers stay thin — validation/business rules stay in the domain layer, not
      duplicated in the HTTP layer

**Passed review 2026-08-23.** Final shape: `POST /task`, `GET /task/{id}`, `GET /tasks`,
`POST /task/{id}/priority`, `POST /task/{id}/status`, `POST /task/{id}/project_id`, all
against an `Arc<AppState>` holding `Mutex<HashMap<TaskId, Task>>`. Every handler maps a
poisoned lock to `500` via `.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?` rather than
`.expect()`-ing it. Two known, deliberately deferred rough edges, tracked here rather than
silently dropped: (1) `set_status`'s invalid-transition case returns `405 Method Not
Allowed`, which isn't the right semantic fit (`409 Conflict` is) — left as-is since (2)
below will likely touch this same code anyway; (2) `TaskError` still conflates two
unrelated failure domains (`Task::new`'s `TitleMissing` and `Task::set_status`'s
`InvalidStatusChange`), forcing callers of either function to exhaustively match a
variant they can never actually produce — worth splitting into two error types
(colocated with their owning functions, same pattern as `StatusError`) next time this
area is touched.

## Phase 3 — Persistence

**Scope:** replace the `Mutex<HashMap<TaskId, Task>>` placeholder with a real repository
seam. This is the first phase where a trait/port is actually justified — there are about
to be two implementations (the real adapter, and something swappable for tests), which is
exactly the condition the project's "ports appear only when an adapter needs them"
principle has been waiting for.

- Define a repository trait capturing the operations handlers actually call today
  (insert, get, get_mut-equivalent update, list) — derive its shape from
  `handlers.rs`'s existing usage, not from speculation about what a repository
  "should" support.
- Implement it against a real datastore (Postgres via `sqlx` is the conventional
  default for this kind of service; open to alternatives if you have a reason).
- `AppState` holds the repository (behind the trait, likely `Arc<dyn Repository>` or
  a generic parameter) instead of the raw `Mutex<HashMap<_, _>>` directly.
- Handlers change minimally — they already only touch the map through a small
  set of operations, so this should mostly be swapping what's behind those calls,
  not rewriting the handlers themselves.

**Explicitly out of scope for Phase 3:** auth, gRPC, the `TaskError`/status-code cleanup
noted above (still tracked, not forgotten — just not this phase's focus).

**Acceptance criteria (draft — refine once the trait shape is underway):**

- [ ] Repository trait exists with a real adapter implementation; handlers depend on
      the trait, not on `HashMap`/`Mutex` directly
- [ ] Data survives a process restart (proof the in-memory placeholder is actually gone)
- [ ] Persistence failures (connection errors, query errors) map to sensible HTTP
      status codes, not a panic or blanket 500 for everything
- [ ] `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` clean
- [ ] No trait method exists that the real adapter doesn't need (no speculative surface)

**Mid-phase course correction (2026-08-30):** the original trait shape had field-specific
mutators (`set_priority`, `set_status`, `set_project_id`) living directly on
`TaskRepository`. That's wrong: it pushes business validation (`Task::set_status`'s
`transition_to` rules) into every adapter individually rather than enforcing it once.
Concretely, `InMemoryTaskRepository::set_status` happened to route through
`Task::set_status` correctly, but nothing about the trait *required* that — a
`PostgresTaskRepository::set_status` written as a raw `UPDATE tasks SET status = $1`
would silently accept invalid transitions the in-memory adapter rejects, since it never
touches the domain method at all. Two adapters silently disagreeing about what's a legal
state transition is a real correctness bug, not a style issue.

**Fix:** collapse `TaskRepository` down to a pure persistence port — `get_task`,
`get_tasks`, `create_task`, `save_task(&self, task: &Task)` — no field-specific mutators.
Business logic moves to a new `TaskService` layer (standard DDD/hexagonal "application
service") sitting between `handlers.rs` and `TaskRepository`: it loads a `Task`, calls the
domain's own validated methods (`set_status`, `set_priority`, ...), and persists the
result via `save_task`. This guarantees `transition_to`'s validation runs exactly once,
in the domain layer, regardless of which repository implementation is behind the trait —
no adapter has a status-specific write path left to get wrong. Handlers become thin
callers of `TaskService`, not direct callers of `TaskRepository`.

**Known, deliberately deferred limitation:** load-then-save is two round trips with a
window in between, unlike the atomic `UPDATE ... RETURNING` the old field-specific
methods had. Two concurrent requests modifying the same task could race — the second
`save_task` can silently overwrite what the first wrote (a "lost update"), since nothing
currently checks that the row hasn't changed since it was loaded. Acceptable for this
project's scope (no real concurrent traffic), but a known gap, not an oversight. Real
fix, if revisited: optimistic concurrency via a `version` column
(`UPDATE ... WHERE task_id = $1 AND version = $2`, treat zero rows affected as a
conflict — the DB-row equivalent of compare-and-swap) or Postgres `SERIALIZABLE`
isolation with app-level retry on serialization failure; the HTTP-level equivalent of
the same idea is `ETag`/`If-Match` with `412 Precondition Failed`. Pessimistic locking
(`SELECT ... FOR UPDATE` in a transaction) and event sourcing (Akka Persistence-style
append-only event log, conflict detected via expected-vs-actual stream version) are the
next tiers up if conflict frequency or auditability ever demanded it — neither is
warranted here.

**Finalized shape (2026-09-19):**
- `TaskRepository` (pure persistence port): `get_task(id) -> Option<Task>`,
  `get_tasks() -> Vec<Task>`, `create_task(Task) -> TaskId`,
  `update_task(&Task) -> ()` — the last one replaces `set_priority`/`set_status`/
  `set_project_id`, taking a whole already-validated `Task` and writing every column.
  `get_task` and `get_tasks` stay as two separate methods rather than collapsing
  into one — they're different queries (`WHERE task_id = $1`, indexed, vs. a full
  table scan), and `Option<Task>` is the correct, precise type for "at most one row
  by primary key" — collapsing it into `Vec<Task>` would both weaken that type
  guarantee and force every single-task lookup (including every `TaskService`
  mutator, which needs to load one task before mutating it) through a full-table
  fetch.
- `TaskService` (application/orchestration layer): `create_task`, `set_priority`,
  `set_status`, `set_project_id` — each of the three setters loads via
  `repository.get_task`, mutates via the domain's own validated method
  (`Task::set_priority`/`set_status`/etc., so `transition_to`'s rules run exactly
  once regardless of adapter), then persists via `repository.update_task`.
  `create_task` included on the service (not left as a direct
  handler → `Task::new` → `repository.create_task` call) purely for consistency —
  handlers should always go through one seam, not sometimes the service and
  sometimes the repository directly — even though creation itself has no existing
  state to load/mutate.
- No `delete`/`remove` on either layer yet — deferred deliberately, not an
  oversight. Most of the everyday "remove a task" need is already covered by
  `Status::Cancelled` as a terminal state (matches how Jira/Linear/Asana mostly
  handle this in practice); true deletion is a rarer, more administrative
  operation. When it's added, it needs its own deliberate choice between hard
  delete (`DELETE FROM tasks`) and soft delete (a `deleted_at`/`is_deleted` column,
  preserving the row for audit/undo — the more common production choice) rather
  than defaulting to whichever is easiest to bolt on.
- `TaskService` gets its own error type composing both `RepositoryError`
  (persistence failures) and domain validation failures (`TaskError`, from
  `transition_to`) — collapsing both into one generic service-level error would
  reintroduce the "blanket 500 for everything" problem already fixed once at the
  HTTP layer (a rejected transition should still map to `409`, not `500`).

## Decision Log

- 2026-08-09 — Chose Axum (HTTP) before tonic (gRPC) for the API phase: closer to
  what most Rust job interviews and day-one work expect; gRPC added once the domain
  and REST layer are solid.
- 2026-08-09 — Deferred the repository trait to Phase 3 rather than defining it in
  Phase 1: avoids designing a port before an adapter exists to justify its shape.
- 2026-08-21 — Kept error message fields as `String` rather than `&'static str`:
  matches convention seen in most codebases and keeps the door open for
  interpolated (non-static) message content later. Consequence: message content
  can't be checked with a literal in a `matches!`/`match` pattern — needs a
  `matches!(.., if message == "...")` guard or an explicit `match`/`assert_eq!`
  instead.
- 2026-08-29 — Settled the Phase 3 concurrency/state shape: `TaskRepository: Send +
  Sync` (required because `Arc<T>` is only `Send + Sync` when `T` is, which
  transitively determines whether `AppState` — and therefore each handler's
  `Future` — is `Send`, which axum's `Handler` bound requires for tokio's
  multithreaded runtime to move in-flight futures between worker threads);
  `InMemoryTaskRepository` wraps `Mutex<HashMap<TaskId, Task>>` internally rather
  than `AppState` wrapping the repository in its own outer lock (interior
  mutability belongs inside each adapter, not imposed on every adapter by the
  caller — see also the earlier call not to double-wrap with an outer `Mutex`);
  `AppState` holds one field, `repository: Arc<dyn TaskRepository>`, derives
  `Clone`, and is passed to `with_state` directly (no redundant outer
  `Arc<AppState>`) since cloning `AppState` already reduces to one `Arc` refcount
  bump. `#[async_trait]` used to make `TaskRepository`'s `async fn` methods
  dyn-compatible (bare `async fn` in a trait isn't, since each impl's generated
  future is a distinct, differently-sized anonymous type — incompatible with a
  vtable, which needs uniform method signatures across implementations).
