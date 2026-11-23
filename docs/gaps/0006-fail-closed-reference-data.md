# 0006 — Missing reference data fails open (no band / freeze / reference price / equity skips the check)

**Status**: open
**Filed**: 2026-11-23 — workspace design review (root plan 0021) · **Kind**: design-flaw · **Severity**: high
**Plan**: [0003](../plans/0003-design-review-2026-11.md)

## What's stubbed or hardcoded

Every limit check that needs reference data is skipped silently when that data is absent:

- `src/engine.rs:~641-647` and `1098-1104`: the price band is checked only `if let Some(band)`; with no band any limit price passes.
- `src/engine.rs:~700-713`: the notional check uses `.unwrap_or(Price::ZERO)` when a market order has no band, so notional is 0 and `max_notional_per_order` always passes.
- `src/engine.rs:~716-728`: the freeze check is skipped when `freeze_limits` has no entry. `freeze_limits: HashMap<InstrumentId, Quantity>` (`src/limits.rs:67`) defaults to empty (serde `default`). The price collar is skipped when there is no band or `reference == 0`.
- `src/engine.rs:1155-1226`: `check_position_concentration` / `check_mis_leverage` return `Ok` when `total_equity <= 0`.
- `tantra-runtime/crates/tantra-runtime-api-rest/src/state.rs:247` builds the REST gate with `RiskEngine::default()` (no limits at all) plus `register_algo(REST_ALGO_ID)`.

## Why

An F&O order for an instrument whose band/freeze data was never loaded (new contract, loader failure, stale refresh) is approved with zero price, quantity or notional protection. A fat-finger market order passes the notional cap because the reference price is missing. The REST gate as built is effectively an algo-registry check only. This contradicts the "AI cannot bypass the gate" invariant and the fail-closed stance of `.agents/rules/security-and-secrets.md`.

## How to close it

1. Add `RiskConfig::strictness` (`FailClosed` default for live, `Permissive` opt-in for tests/paper).
2. Add refusals `MissingReferenceData { instrument, field }` for band / freeze / reference price whenever the corresponding limit is configured.
3. Treat `total_equity <= 0` as a refusal in `check_position_concentration` and `check_mis_leverage`.
4. Add an `as_of` stamp to `PriceBand` and a `StaleReferenceData` refusal.
5. Make `RiskEngine::default()` unusable for live: provide `RiskEngine::unconfigured()` that refuses everything, or a constructor that requires validated `RiskLimits` (see plan R2 for `try_new`). Update `state.rs:247` to use it.
6. Keep every new check branch allocation-free so the sub-15 µs SLA holds (flag lookups only, no formatting on the approve path).

**Verification**: `tests/fail_closed_tests.rs` with one test per missing input; a proptest that every unconfigured instrument is refused when strict; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `make check`, `cargo test -p tantra-risk --profile prod`, `cargo bench` before/after.

## Related

- Closed gaps 0001/0002 covered only collar and reduce-only enforcement.
- Root gap 0013 (pre-trade gate not single, shared or fail-closed).
- [0010](0010-shared-risk-state-and-kill-switch.md) (REST/actors/router each build their own engine); [0012](0012-decision-exchange-and-algo-id-typing.md) (wrong-exchange keys hit this fail-open path).
