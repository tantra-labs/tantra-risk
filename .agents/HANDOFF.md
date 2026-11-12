# HANDOFF — vyapaar-risk

**Status**: Docs batch complete — former `docs/roadmap/` dismantled into
`docs/design/` + `docs/gaps/` + `docs/plans/`; all design docs rewritten to match source.
**Date**: 2026-11-12 · Repo: `vyapaar-risk` · Branch: `main`

## Phase / Slice Status

- **Docs rewritten against source** (docs-only, no `.rs` touched): `architecture.md` and all
  `code-*.md` reference docs now state actual structs/enums/signatures; the incorrect
  `ReduceOnly`/`Halted` on `AlgoRegistrationStatus`, the non-existent `algo_id` format regex,
  the fake `risk_refusals`/`risk_approvals`/`algo_halted`/`kill_switch_activated` counters,
  the non-existent `global_kill_switch()` method, and the `Emergency` tool hint are all gone.
- **Gaps filed (source-verified)**: 0001 `price_collar_bps` unenforced · 0002 reduce-only not
  enforced · 0003 `check_order_fast` skips per-scrip OTR check / warning zone / cooldown
  auto-resume · 0004 `window_1min_ms`/`window_5min_ms` config knobs never consulted.
- **Plans**: 0001 (tool-call annotation hooks, commit `5b00cda`) kept in Implemented table;
  new `0002-close-declared-unenforced-risk-controls` collects gaps 0001–0004.
- **`docs/roadmap/` removed**; completed milestones folded into
  `docs/design/implementation-record.md` (test inventory + verification commands) and
  `code-migration-map.md` (honba-mon → vyapaar-risk mapping kept).

## Decisions & Discoveries

- `vyapaar-exec/src/executor.rs::RiskGatedExecutor` and `vyapaar-exec/src/router/src/otr.rs`
  do not exist (verified `vyapaar-exec/src/` absent) — all such cross-references dropped.
- `RiskRefusal::ReduceOnlyViolation` is never constructed; `GlobalState::ReduceOnly` checked
  nowhere; `window_otr_1min`/`window_otr_5min` hardcode 60s/300s (config fields dead).
- `Instant::now()` in `engine.rs` is latency-metrics only; no `Utc::now()`/`SystemTime::now()`
  in `src/`.

## Immediate Next Step

Implement plan 0002, phase 1 (gap 0002, high severity): enforce `GlobalState::ReduceOnly` in
`evaluate_core_checks` + `check_order_fast_checks` with `RiskRefusal::ReduceOnlyViolation`,
TDD with a failing test in `tests/` first. Then gaps 0001 → 0003 → 0004.

## Operating Invariants

- Canonical profile `prod`: `cargo build/check/test --profile prod`.
- Tests live in `tests/` parallel to `src/`; no in-file `#[test]` blocks.
- Zero-I/O domain purity: no `tokio`, no network/file I/O, no `Utc::now()` in `src/`.
- Zero floats: all notional, price-band, loss, OTR calculations use `rust_decimal::Decimal`.
- Latency SLO: `p99.99 < 15µs` via zero-allocation `RiskEngine::check_order_fast`.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` green.
