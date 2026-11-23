# 0015 — Latency SLA is documented but not verified in CI; the allocation claim is false on the main path

**Status**: open
**Filed**: 2026-11-23 — workspace design review (root plan 0021) · **Kind**: deployment · **Severity**: medium
**Plan**: [0003](../plans/0003-design-review-2026-11.md)

## What's stubbed or hardcoded

- `src/metrics.rs:1-12` states p99.99 < 15 µs, 50k orders/s and "zero heap allocations".
- `src/engine.rs:~463` builds `String` trace ids, a `tracing::info_span!` and `format!` per call in `evaluate_core` (allocates); refusals allocate `String`s (`algo_id.to_string()`).
- `benches/risk_benchmarks.rs` uses Criterion with a constant `now`, no thresholds and no regression gate.
- `.github/workflows` at the workspace root has no risk job and `tantra-risk` has no `.github`; no CI covers the repo.
- No `cargo check --target wasm32-unknown-unknown` is run (see plan R1: `prometheus` and `Instant` prevent a clean wasm build).

## Why

The headline contract of the crate is unenforced: any change (including those proposed in [0006](0006-fail-closed-reference-data.md) to [0014](0014-otr-accounting-order.md)) can silently regress latency. The allocation claim is false on the `check_order` / `check_decision` path, and span creation alone costs several microseconds.

## How to close it

1. Add a CI job running `cargo bench -- --save-baseline` with a compare gate (`critcmp`) and a fail threshold; benches must advance `now` (see [0008](0008-bounded-rate-windows.md)).
2. Add a counting-allocator (`dhat`) test in `tests/` asserting zero allocations on the `check_order_fast` approve path.
3. Move tracing span creation behind `tracing::enabled!` and avoid per-call `String` building on the approve path.
4. Document that `check_order_fast` is the SLA path and route REST and actors to it.
5. Add the wasm32 target check to the job.

**Verification**: the CI job itself (fmt, clippy `-D warnings`, `make check`, targeted tests with `--profile prod`, bench compare gate, wasm32 check); `tests/alloc_free_fast_path_tests.rs`.

## Related

- Root gap 0024 (no CI for 7 Rust repos), root plan 0023 (build/CI/release engineering); root plan 0019 P3.2 (pipelines), P1.4 (`Instant`).
- [0008](0008-bounded-rate-windows.md).
