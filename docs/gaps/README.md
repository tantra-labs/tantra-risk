# Gaps Index — `tantra-risk`

Per workspace rule `.agents/rules/gap-tracking.md`:

- Gaps are individual missing features / declared-but-unenforced controls / stubs
  (`docs/gaps/XXXX-*.md`).
- Plans are collections of gaps (`docs/plans/XXXX-*.md`).
- When implemented and verified, a gap document is removed from this index and recorded
  as Closed (with the closing commit/PR) in the table below.

This directory was created when the former `docs/roadmap/` was dismantled: completed
milestones moved to `docs/design/implementation-record.md` and the file mapping to
`docs/design/code-migration-map.md`; remaining declared-but-unenforced items were filed here
as gaps. Every gap below was verified against `src/` on the filing date (2026-11-12).

## Open Gaps

| # | Title | Severity | Status | Plan |
|---|-------|----------|--------|------|
| [0006](0006-fail-closed-reference-data.md) | Missing reference data fails open (no band / freeze / reference price / equity skips the check) | High | open | [0003](../plans/0003-design-review-2026-11.md) |
| [0007](0007-modify-order-bypasses-halts.md) | `ModifyOrder` bypasses global halt, algo halt, registry and market-circuit checks | High | open | [0003](../plans/0003-design-review-2026-11.md) |
| [0008](0008-bounded-rate-windows.md) | Unbounded `order_history`; O(n) OTR window scans | High | open | [0003](../plans/0003-design-review-2026-11.md) |
| [0009](0009-trading-day-rollover.md) | No trading-day rollover; sticky global daily-loss halt | High | open | [0003](../plans/0003-design-review-2026-11.md) |
| [0010](0010-shared-risk-state-and-kill-switch.md) | Per-process, multiplied risk state; kill switch not shared or persisted | High | open | [0003](../plans/0003-design-review-2026-11.md) |
| [0011](0011-encapsulated-state-and-operator-audit.md) | `pub` mutable engine state; unaudited resume | Medium | open | [0003](../plans/0003-design-review-2026-11.md) |
| [0012](0012-decision-exchange-and-algo-id-typing.md) | Hardcoded `Exchange::Nse`; stringly-typed algo and instrument identity | Medium | open | [0003](../plans/0003-design-review-2026-11.md) |
| [0013](0013-metrics-label-cardinality.md) | Unbounded Prometheus label cardinality from `algo_id` / scrip | Medium | open | [0003](../plans/0003-design-review-2026-11.md) |
| [0014](0014-otr-accounting-order.md) | Refused orders inflate OTR; scrip breach halts whole algo; slice ceil bug | Medium | open | [0003](../plans/0003-design-review-2026-11.md) |
| [0015](0015-latency-sla-ci-gate.md) | Latency SLA and allocation claim not verified in CI | Medium | open | [0003](../plans/0003-design-review-2026-11.md) |

## Closed Gaps

| # | Title | Severity | Status | Closed In |
|---|-------|----------|--------|-----------|
| 0005 | No stop-protection (SL/TSL) risk validation — mandatory SL, SL-distance band, SL-within-collar, max-loss-per-strategy, TSL no-widening (Elephant Edge EE+) | High | closed | Closed 2026-11-17 (T12.6, `e9f8148`): `src/trade_plan_gate.rs` — `StopProtectionConfig` + `validate_trade_plan` (mandatory SL, `StopTooTight`/`StopTooWide` band vs ATR/ADR%, `StopOutsidePriceCollar`, `MaxLossExceeded`) + `validate_stop_update` (no-widening TSL on both sides); new `RiskRefusal` variants; `SizingRiskRefusal::{MaxWeightExceeded, MaxAdvParticipationExceeded}` + multi-currency (`Money.currency`) refusal formatting; `RiskEngine::check_trade_plan`/`check_stop_update` + `with_stop_protection`. Tests `trade_plan_gate_tests` (12) + `sizing_gate_weight_adv_tests` (7) + `trade_plan_gate_proptest` (2). |
| 0001 | Market-order price collar (`price_collar_bps`) declared but never enforced | Medium | closed | Commit `9a0bc59` (`feat(risk): enforce reduce-only, price collar, fast-path OTR parity, OTR windows (plan 0002)`) |
| 0002 | Reduce-only mode (`GlobalState::ReduceOnly`) not enforced by any check path | High | closed | Commit `9a0bc59` |
| 0003 | `check_order_fast` skips per-scrip OTR check, warning zone, and cooldown auto-resume | Medium | closed | Commit `9a0bc59` |
| 0004 | OTR window config knobs (`window_1min_ms` / `window_5min_ms`) never consulted | Low | closed | Commit `9a0bc59` |

## Verification

Each gap was filed only after grepping `src/` for the exact identifiers. There are no
`TODO` / `FIXME` / `stub` / `unimplemented` markers anywhere in `src/` (verified
2026-11-12); the gaps below are behavioral — declared configuration or API surface that no
check path actually enforces. All four behavioral gaps were closed 2026-11-13 under plan
0002 with TDD tests in `tests/plan0002_tests.rs`.
