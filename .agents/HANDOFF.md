# HANDOFF — tantra-risk

**Status**: Gap 0006 (fail-closed reference data) closed 2026-11-28 (`9bfb3c2`); plan 0003
(design review 2026-11) gaps 0007–0015 open.
**Date**: 2026-11-28 · Repo: `tantra-risk` · Branch: `main`

## Phase / Slice Status

- **Gap 0006** (`9bfb3c2`): `RiskLimits::strictness` (`FailClosed` default / `Permissive`),
  `max_reference_age_ms`, `MissingReferenceData` / `StaleReferenceData`, `RiskEngine::unconfigured()`,
  one shared reference-rules helper for `check_order` + `check_order_fast`. Needs tantra-core
  `PriceBand::as_of` (`b2cef47`).
- Plan 0002 (gaps 0001–0004: reduce-only, collar, fast-path OTR parity, OTR windows) shipped in `9a0bc59`.

## Decisions & Discoveries

- FailClosed is **scoped** (user decision 2026-11-27): only rules that need data demand it —
  collar/notional market orders need a band, NFO/MCX/CDS need a freeze entry, stale/unstamped bands are
  refused when `max_reference_age_ms` is set, concentration/leverage need `total_equity > 0`. A limit
  order with no band and no such rule still passes.
- Downstream: tantra-runtime's paper REST gate and backtest executor run `Permissive` explicitly until a
  band/freeze loader exists (tantra-runtime gap 0039). A `--risk-config` TOML without `strictness`
  means FailClosed.
- Gap 0012 interaction: `check_decision` keys every `PlaceOrder` as NSE, so NFO decisions skip the F&O
  freeze-entry rule until 0012 lands.
- Reduce-only is position-free by design (close-long approximation); see `docs/design/code-reduce-only.md`.
- `check_order_fast` stays zero-allocation; bench ~3.7 µs per check (2026-11-27).

## Immediate Next Step

- Gap 0007 (modify-order bypasses halts), then the rest of plan 0003.

## Operating Invariants

- Canonical profile `prod`: `cargo build/check/test --profile prod`.
- Tests live in `tests/` parallel to `src/`; no in-file `#[test]` blocks.
- Zero-I/O domain purity: no `tokio`, no network/file I/O, no `Utc::now()` in `src/`.
- Zero floats: all notional, price-band, collar, loss, OTR calculations use `rust_decimal::Decimal`.
- Latency SLO: `p99.99 < 15µs` via zero-allocation `RiskEngine::check_order_fast`.
- `cargo fmt --check`, `make clippy`, `make test`, `make prod` green.
