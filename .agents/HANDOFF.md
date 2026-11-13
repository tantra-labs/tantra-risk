# HANDOFF — tantra-risk

**Status**: Plan 0002 complete — all four declared-but-unenforced risk controls
implemented, tested, and closed (gaps 0001–0004).
**Date**: 2026-11-13 · Repo: `tantra-risk` · Branch: `main`

## Phase / Slice Status

- **Plan 0002 shipped** (commit `9a0bc59`): 
  - **Reduce-only enforced (0002, High)**: `GlobalState::ReduceOnly` refuses
    order-opening BUYs on both `evaluate_core_checks` and `check_order_fast_checks`
    (`RiskRefusal::ReduceOnlyViolation`), refuses `ModifyOrder` and new position sizing
    (`SizingRiskRefusal::ReduceOnlyViolation`), allows reducing SELLs and cancels.
  - **Price collar enforced (0001, Med)**: `RiskLimits::price_collar_bps` applied on both
    paths — market orders whose worst-case band-edge fill exceeds `reference ± collar`
    → new `RiskRefusal::PriceCollarViolation`.
  - **Fast-path OTR parity (0003, Med)**: `check_order_fast` now records-and-checks the
    per-scrip OTR before approval (`ScripOtrBreached`), logs the OTR warning zone, and
    auto-resumes halted algos after `halt_cooldown_ms`.
  - **OTR windows live (0004, Low)**: `window_otr_1min`/`window_otr_5min` read
    `OtrConfig::{window_1min_ms, window_5min_ms}`; engine enforces the configured
    short-term window (5-min stays analysis-only, documented).
- **API change**: `check_order_fast` gains required `side: OrderSide` (side-aware reduce-only
  and collar). FFI (`tantra-ffi`) updated in lockstep: pyclass + codegen stub + `_tantra.pyi`
  + `python/tests/test_risk.py` (new reduce-only test); `cargo check` green there.
- **Docs**: gap docs 0001–0004 and plan 0002 removed from `docs/{gaps,plans}/`; README
  indexes moved rows to Closed/Implemented (closing commit `9a0bc59`); `docs/design/*`
  rewritten to match the enforced behavior.

## Decisions & Discoveries

- Reduce-only is **position-free by design**: the gate has no position book, so "reduces
  risk" is the documented close-long approximation (SELL passes / BUY refused). Position-aware
  reduction is enforced upstream by `tantra-runtime-api-rest::orders::state_refusal`
  (ADR 0018). Full decision record: `docs/design/code-reduce-only.md`; workspace ADR:
  `docs/design/0002-reduce-only-contract.md` (repo root).
- `check_order_fast` intentionally stays zero-allocation (only branch + existing
  `HashMap::get_mut`); the Prometheus `set_otr_ratio` gauge remains full-path-only.
- Collar semantics: the pure gate cannot know a market order's fill, so the worst-case
  band-edge fill (band.upper for BUY / band.lower for SELL) is collared against the band
  reference price; skipped when no band or a zero reference exists.
- `tantra-runtime-strategy/tests/runner_risk.rs` references `RiskRefusal::{InstrumentUnknown,
  ReduceOnly, PriceCollar, TradingHalted}` which never existed — pre-existing broken test
  binary (matches root T0.6 "runtime test-binary compiles OOM"); untouched.

## Immediate Next Step

- None in `tantra-risk` — T1 track (plan 0002) is complete and the gaps index is clean.
- Optional future: wire a 5-minute OTR enforcement tier (currently analysis-only) and a
  position-aware reduce-only mode when a position book is available upstream.

## Operating Invariants

- Canonical profile `prod`: `cargo build/check/test --profile prod`.
- Tests live in `tests/` parallel to `src/`; no in-file `#[test]` blocks.
- Zero-I/O domain purity: no `tokio`, no network/file I/O, no `Utc::now()` in `src/`.
- Zero floats: all notional, price-band, collar, loss, OTR calculations use `rust_decimal::Decimal`.
- Latency SLO: `p99.99 < 15µs` via zero-allocation `RiskEngine::check_order_fast`.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --profile prod` green.
