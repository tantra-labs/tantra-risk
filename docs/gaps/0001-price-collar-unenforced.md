# 0001 — Market-order price collar (`price_collar_bps`) declared but never enforced

**Status**: open

## What's stubbed or hardcoded

`RiskLimits::price_collar_bps: Option<u32>` (`src/limits.rs`) is declared and documented as
the "allowed market order deviation collar in basis points (1 bp = 0.01%)". It is validated
in `RiskLimits::is_valid()` (rejects `Some(0)`), but **no code path in `src/engine.rs` ever
reads the field**: neither `evaluate_core_checks` nor `check_order_fast_checks` consults it.
For a market order (`limit_price == None`) the engine skips the price-band check entirely and
evaluates notional at the band reference price (`Price::ZERO` fallback), so a market order
deviating arbitrarily far from the last traded / reference price is not collared.

Source-verified: `price_collar_bps` appears only in `src/limits.rs` (field + `is_valid()`);
zero references in `src/engine.rs`.

## Why

The `price_collar_bps` knob exists specifically to cap market-order price deviation (a
market-microstructure guard: a market order must not execute far from the reference price
after a data glitch or stale feed). Today the knob is dead configuration — an operator who
sets it gets no protection, and the validation in `is_valid()` falsely implies enforcement.

## How to close it

1. In `evaluate_core_checks` and `check_order_fast_checks`, when `limit_price.is_none()` and
   `self.limits.price_collar_bps` is set, compute a collar window around the band reference
   price (`self.price_bands[instrument].reference_price`) and reject market orders whose
   implied deviation would exceed the collar — either by refusing
   (`RiskRefusal::PriceBandViolation` or a new collar-specific refusal variant) or by
   requiring a derived limit price.
2. Add tests in `tests/` (Red → Green): a market order within the collar passes; a market
   order beyond the collar is refused; collar behavior interacts correctly with the
   notional-at-reference-price evaluation.
3. Keep `RiskLimits::is_valid()` semantics (non-zero collar) aligned with the enforcement.

Related: collected under `docs/plans/0002-close-declared-unenforced-risk-controls.md`.
