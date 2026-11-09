# HANDOFF — vyapaar-risk

**Status**: Migration complete, all milestones done, new documentation batch added: `docs/gaps/` (none yet), `docs/plans/` (plan 0001 implemented), `docs/design/` (migration map + 8 code reference docs), `docs/roadmap/` (updated + migration verification). Batch done.
**Date**: 2026-11-09 · Repo: `vyapaar-risk` · Branch: `main`

## Phase / Slice Status (COMPLETE — merged)

- **All migration milestones complete**: `algo_registry`, `state`, `limits`, `error`, `refusal`, `otr`, `engine`, `sizing_gate`, `metrics`, `tool_hints` — all ported from `honba-mon`.
- **Plan 0001** (Tool-Call Annotation Hooks for MCP Gateway): implemented — `ToolRiskHint`, `risk_classification`, `tests/tool_hints_tests.rs`, commit `5b00cda`.

## Decisions & Discoveries

- `docs/migration-guide.md` decimated: file-by-file migration map moved to `docs/design/code-migration-map.md`; verification commands moved to `docs/roadmap/migration-verification.md`; original `migration-guide.md` removed.
- Code reference docs added: `code-engine.md` (`RiskEngine`, SLOs), `code-otr.md` (sliding-window OTR), `code-limits.md` (`RiskLimits`, freeze limits), `code-algo-registry.md` (SEBI Algo ID registry), `code-sizing-gate.md` (`validate_sizing_request`), `code-metrics.md` (`RiskMetrics`), `code-refusal-state.md` (`RiskRefusal`, `AlgoState`, `GlobalState`), `code-tool-hints.md` (MCP tool risk classification).

## Immediate Next Step

None — no active plans remain in `docs/plans/`. Migration is fully complete.

## Operating Invariants

- Canonical profile `prod`: `cargo build/check/test --profile prod`.
- Tests live in `tests/` parallel to `src/`; no in-file `#[test]` blocks.
- Zero-I/O domain purity: no `tokio`, no network/file I/O, no `Utc::now()` in `src/`.
- Zero floats: all notional, price-band, loss, OTR calculations use `rust_decimal::Decimal`.
- Latency SLO: `p99.99 < 15µs` via zero-allocation `RiskEngine::check_order_fast`.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` green.
