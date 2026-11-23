# 0012 — `check_decision` hardcodes `Exchange::Nse` and uses unvalidated stringly-typed algo and instrument identity

**Status**: open
**Filed**: 2026-11-23 — workspace design review (root plan 0021) · **Kind**: design-flaw · **Severity**: medium
**Plan**: [0003](../plans/0003-design-review-2026-11.md)

## What's stubbed or hardcoded

- `src/engine.rs:~376`: `InstrumentId::new(Exchange::Nse, symbol.clone())` for every `Decision::PlaceOrder`. Freeze and band maps are keyed by `InstrumentId`, so BSE / NFO / MCX decisions look up the wrong key and fall into the fail-open path of [0006](0006-fail-closed-reference-data.md).
- Algo ids are `String` keys everywhere (`src/engine.rs:36-48`, `src/algo_registry.rs:~22`).
- `AlgoIdRegistry::is_registered_and_active` returns `Result<(), String>` (the typed-error item is already tracked as root plan 0019 P3.3; listed here for context).
- `src/metrics.rs`: `algo_id` and `scrip` are Prometheus labels fed from request data (see [0013](0013-metrics-label-cardinality.md)).

## Why

An AI decision for a non-NSE symbol, or a segment-qualified one, bypasses configured freeze limits and bands without any error. Free-form algo strings can be empty, mis-cased or arbitrarily long, and they key unbounded maps.

## How to close it

1. Provider change in `tantra-core`: require `Decision::PlaceOrder` to carry a full `InstrumentId`, and introduce a validated `AlgoId` newtype (tracked as tantra-core plan 0006 R5, stringly-typed ids).
2. Until that lands, refuse any `Decision::PlaceOrder` whose exchange is ambiguous rather than defaulting to NSE.
3. Re-key `RiskEngine` algo state, registry and trackers on `AlgoId`; return typed errors from the registry.
4. Interning or a fixed-size id keeps hash/compare cost unchanged so the sub-15 µs path is not regressed.

**Verification**: `tests/decision_identity_tests.rs` (BSE/NFO/MCX decision uses its own freeze and band entries; ambiguous exchange is refused; invalid `AlgoId` rejected at construction); proptest on `AlgoId` validation; `cargo clippy --all-targets -- -D warnings`, `cargo test -p tantra-risk --profile prod`.

## Related

- Provider repo: `tantra-core` (Decision shape, `AlgoId` newtype); tantra-core plan 0006 R5.
- [0006](0006-fail-closed-reference-data.md), [0007](0007-modify-order-bypasses-halts.md), [0013](0013-metrics-label-cardinality.md).
