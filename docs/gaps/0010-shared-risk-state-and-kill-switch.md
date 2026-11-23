# 0010 — Risk state is per-process and multiplied: several independent `RiskEngine` instances, kill switch not shared, nothing persisted

**Status**: open
**Filed**: 2026-11-23 — workspace design review (root plan 0021) · **Kind**: deployment · **Severity**: high
**Plan**: [0003](../plans/0003-design-review-2026-11.md)

## What's stubbed or hardcoded

- `tantra-runtime/crates/tantra-runtime-api-rest/src/state.rs:182`: `risk_engine: Arc<Mutex<RiskEngine>>` (REST).
- `tantra-runtime/crates/tantra-runtime-actors/src/risk.rs:21`: `RiskActor` owns another engine by value; `execution.rs:87` holds another `Arc<Mutex<..>>`.
- `tantra-exec/crates/tantra-exec-router/src/lib.rs:50-61`: `ResilientRouter.risk_engine` builds yet another with `RiskEngine::new`.
- `tantra-ffi/crates/tantra-ffi-py/src/risk.rs` and `runner.rs` each wrap their own; enhancer crates take `RiskEngine` by value (`tantra-runtime-enhancer/src/lib.rs:119+`).
- `tantra-runtime/crates/tantra-runtime-actors/src/risk.rs:256-261`: `HaltGlobal` mutates only that actor's engine (timestamp from `Utc::now()` at the adapter, which is correct), so the REST engine and the router engine stay Active.
- `tantra-risk` has no persistence or serialization of halt state: `GlobalState` derives serde, but `RiskEngine` is not `Serialize` and has no snapshot/restore API.
- No health/readiness accessor exposes gate state (halted, limits loaded, policy version).

## Why

A kill switch fired through one path (actor, FFI, REST) does not stop orders on another path in the same process. With N replicas, OTR counts, `order_rate_limit`, daily loss and algo halts are each per replica, so effective limits are N times those configured. A restart clears halts: restart equals resume trading. This breaks `.agents/rules/scalability-and-concurrency.md` section 4 (restart-safe trading state).

## How to close it

1. Split `RiskEngine` into a pure stateless checker `fn check(&RiskState, &RiskConfig, &Order, now) -> Verdict` and a `RiskStateStore` port holding halts, counters and loss. Provide an in-process implementation here and a shared/persistent one in `tantra-runtime-persistence`.
2. Inject a single composition-root instance as `Arc<dyn RiskGate>` into REST, actors, router and FFI.
3. Add a serde `RiskSnapshot` with restore on boot; halts default to the more restrictive of persisted and newly configured state.
4. Add a `RiskStatus` accessor for health/readiness.
5. Decide and record an ADR: one central gate service or a shared state store when running N replicas; measure any out-of-process hop against the 15 µs in-process target. The in-process hot path must stay lock-light and allocation-free.

**Verification**: `tests/snapshot_restore_tests.rs` (halt survives snapshot/restore; restrictive merge); proptest that restore is idempotent; a cross-path test in `tantra-runtime` showing a halt through one handle refuses on every path; `cargo bench` (SLA regression), `make check`, `make prod`.

## Related

- Root gap 0013, root plan 0022 (deployment architecture, HA); root plan 0019 P3.2/P3.3 (structure only).
- [0011](0011-encapsulated-state-and-operator-audit.md) (control vs gate traits), [0009](0009-trading-day-rollover.md).
