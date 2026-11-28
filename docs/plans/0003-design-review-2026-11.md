# 0003 — tantra-risk design review (2026-11): fail-closed gating, bounded state & shared persistent limits

**Status**: Proposed (2026-11-23)
**Scope**: `tantra-risk` · **Umbrella**: root `docs/plans/0021-workspace-design-review-2026-11.md`

## 0. Executive Summary & Objective

The design review found that the gate is correct on its happy path but fails open when reference data is missing, lets `ModifyOrder` skip every state check, grows without bound when no rate limit is configured, never rolls over a trading day, and exists as several unsynchronised per-process instances so a kill switch is not global and a restart resumes trading. The REST gate is built from `RiskEngine::default()` (`tantra-runtime/crates/tantra-runtime-api-rest/src/state.rs:247`), which has no limits.

This plan makes the gate fail-closed, bounded, day-aware, encapsulated and shareable/persistent.

**Binding constraint on every phase and item**: the sub-15 µs p99.99 SLA of the pre-trade check (50k orders/s burst) must hold. Each change is benchmarked before and after against the CI gate from [0015](../gaps/0015-latency-sla-ci-gate.md); any item that cannot meet the budget is redesigned or kept off the hot path (state mutation on commit, observers no-op by default, no allocation on the approve path).

Non-goals: merging the two check pipelines and the typed `Result<_, String>` registry errors and `Instant::now` removal are owned by root `docs/plans/0019-codebase-audit-refactoring-unification-and-wiring.md` (P3.2, P3.3, P1.4); this plan only adds the behavioural fixes on top.

## 1. Context & Gaps Covered

| Gap | Title | Kind | Severity |
|---|---|---|---|
| 0006 (closed `9bfb3c2`) | Missing reference data fails open | design-flaw | high |
| [0007](../gaps/0007-modify-order-bypasses-halts.md) | `ModifyOrder` bypasses halts, registry and circuit checks | design-flaw | high |
| [0008](../gaps/0008-bounded-rate-windows.md) | Unbounded `order_history`; O(n) OTR window scans | design-flaw | high |
| [0009](../gaps/0009-trading-day-rollover.md) | No trading-day rollover; sticky global daily-loss halt | design-flaw | high |
| [0010](../gaps/0010-shared-risk-state-and-kill-switch.md) | Per-process, multiplied risk state; no persistence | deployment | high |
| [0011](../gaps/0011-encapsulated-state-and-operator-audit.md) | `pub` mutable state; unaudited resume | design-flaw | medium |
| [0012](../gaps/0012-decision-exchange-and-algo-id-typing.md) | Hardcoded `Exchange::Nse`; stringly-typed algo ids | design-flaw | medium |
| [0013](../gaps/0013-metrics-label-cardinality.md) | Unbounded Prometheus label cardinality | design-flaw | medium |
| [0014](../gaps/0014-otr-accounting-order.md) | Side effects recorded before later checks reject | design-flaw | medium |
| [0015](../gaps/0015-latency-sla-ci-gate.md) | Latency SLA not enforced in CI | deployment | medium |

Plan-only items (no standalone gap doc):

- **R1 — Injected observer instead of a Prometheus global.** `src/metrics.rs:~156-158` holds `GLOBAL_RISK_METRICS: LazyLock<RiskMetrics>` with `.expect(..)`; `Cargo.toml` depends on `prometheus = "0.13"`; `src/engine.rs:~609`/`~647` convert OTR `Decimal` to `f64` for a gauge (`to_f64().unwrap_or(0.0)`). This breaks the zero-I/O, wasm32-compatible rule and prevents per-engine isolation.
- **R2 — Data-driven, validated limits policy.** `src/otr.rs:~38-52` hardcodes `max_otr = 50`, `min_orders = 20`, 1/5 min windows for all algos/segments; `src/limits.rs:67` is a flat freeze map with no lot size, as-of date or source circular; `RiskLimits::is_valid` (`src/limits.rs:~83-105`) omits `max_quantity_per_order`, `max_daily_loss > 0`, concentration range, `max_mis_leverage > 0`, `warning_threshold <= max_otr`, and `RiskEngine::new` does not call it.
- **R3 — Retained refusal and kill-switch audit trail.** `record_refusal` (`src/engine.rs:~237`) only increments counters; refusals and halt/resume events must reach the hash-chained audit WAL (root gap 0023). Shares the verdict record designed in gap 0011.
- **R4 — De-duplicate freeze catalog, OTR definition, reduce-only and sizing logic across repos.** Freeze limits and slicing: `src/limits.rs:67`, `src/engine.rs:~716`, `suggest_freeze_slices` vs `tantra-exec/crates/tantra-exec-slicer/src/freeze.rs`. OTR: `src/otr.rs` vs `tantra-exec/crates/tantra-exec-router/src/otr.rs` (`min_operations` 25 vs 20). Reduce-only: `src/engine.rs:~520-530` (side == Buy approximation) vs `tantra-runtime-api-rest/src/orders.rs:~659` and `tantra-ffi/python/tantra/risk.py:~68-150`. Sizing: `src/sizing_gate.rs` vs `tantra-ffi/python/tantra/sizing.py`.

## 2. Component Specifications & Phases

### Phase A — Fail-closed correctness and bounded state (quick, low risk)
1. **0006** — `RiskConfig::strictness`, `MissingReferenceData` / `StaleReferenceData` refusals, `RiskEngine::unconfigured()`; switch `state.rs:247` to a configured engine.
2. **0007** — Modify arm checks halts, registry, circuit, band/freeze/notional.
3. **0008** — O(1) running OTR counters; `order_history` only with a rate limit; capacity caps with `CapacityExceeded`.
4. **0013** — label only registered algo ids; drop or cap `scrip` label.
5. **0015** — bench compare gate, allocation-free fast-path test, tracing spans behind `enabled!`, wasm32 check. Land this first within the phase so items 1-4 are measured against the SLA.
**Gate**: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `make check`, `cargo test -p tantra-risk --profile prod`, `cargo bench` (no p99.99 > 15 µs regression), `make prod`.

### Phase B — Structural refactors
1. **0014** — two-phase evaluate/commit, per-scrip halt, uniform `refuse()`, fix slice ceil.
2. **0009** — `TradingDay` (IST) and `roll_session`, per-algo daily loss, halt cause split.
3. **0011** — private fields, `RiskControl` vs `RiskGate` traits, `OperatorId` + `StateChange` audit record.
4. **0012** — full `InstrumentId` / `AlgoId` from `tantra-core`; refuse ambiguous exchange meanwhile.
5. **R1** — `RiskObserver` trait with `NoopObserver` default; move Prometheus to `tantra-runtime` (or an optional feature); latency measured by the caller.
6. **R2** — `RiskPolicy { version, effective_from, source_circular, per_segment }` loaded as data; `RiskLimits::validate()` and `RiskEngine::try_new`.
**Gate**: Phase A gate plus `cargo check --target wasm32-unknown-unknown`, golden vectors for policy parsing and verdict JSON, proptests for `validate()` and state equality on refusal.

### Phase C — Deployment readiness
1. **0010** — pure checker plus `RiskStateStore` port, single `Arc<dyn RiskGate>` composition root, `RiskSnapshot` restore, `RiskStatus` accessor, ADR on central service vs shared store.
2. **R3** — runtime adapter appends verdict/`StateChange` records to the audit WAL.
3. **R4** — single freeze/lot catalog and shared OTR counter type consumed by risk and exec; Python fallback removed in favour of refusal (owned by tantra-ffi).
**Gate**: Phase B gate plus cross-path kill-switch test in `tantra-runtime`, snapshot/restore test, `make test` (use the documented targeted-binary fallback if the `tantra-runtime-api-rest` LTO link OOMs and record the exact command).

## 3. Relationship to Other Plans

- Root plan 0019: P3.2/P3.3 (pipeline merge, typed registry error) and P1.4 (`Instant`) are not duplicated here; this plan sequences behind them where files overlap.
- Root plan 0022 (HA, shared state) consumes 0010 and R3; root plan 0023 (CI) hosts the job from 0015.
- Root gaps 0013 (single fail-closed gate), 0020 (IST trading day), 0023 (audit trail), 0024 (no CI).
- tantra-core plan 0006 R5 provides the `AlgoId` / `InstrumentId` shapes needed by 0012.
- Consumers to update: `tantra-runtime` (REST state, `RiskActor`), `tantra-exec-router`, `tantra-ffi`, `tantra-agent-mcp`.

## 4. Exit Criteria

1. No check silently skips for missing reference data when strictness is `FailClosed`; `RiskEngine::default()` is not constructible for live use.
2. `ModifyOrder` is refused under every halt/registry/circuit state and re-checks limits.
3. Engine memory (deque, trackers) is bounded and proven by test; `window_otr` is O(1).
4. `roll_session` exists and day-scoped state is per algo/account.
5. One gate instance serves REST, actors, router and FFI; halts survive snapshot/restore.
6. Resume and registry mutations require an operator identity and produce audit records.
7. Prometheus is out of the domain crate default build; `cargo check --target wasm32-unknown-unknown` passes.
8. CI enforces the 15 µs p99.99 SLA and an allocation-free fast path.
9. Gaps 0006-0015 are closed and moved to the gaps index with closing commits.
