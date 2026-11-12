# 0002 — Reduce-only mode (`GlobalState::ReduceOnly`) not enforced by any check path

**Status**: open

## What's stubbed or hardcoded

The reduce-only surface exists in three places:

- `GlobalState::ReduceOnly` variant (`src/state.rs`), documented "existing positions can be
  closed/reduced, no new risk".
- `RiskEngine::set_reduce_only()` (`src/engine.rs`) sets `self.global_state = GlobalState::ReduceOnly`.
- `RiskRefusal::ReduceOnlyViolation { details }` (`src/refusal.rs`) with `rule() == "reduce_only_violation"`.

But **no check path rejects on reduce-only**: `evaluate_core_checks` and
`check_order_fast_checks` both match only `GlobalState::Halted` (step 1); `check_decision`
and `RiskEngine::validate_sizing_request` likewise only special-case `Halted`.
`RiskRefusal::ReduceOnlyViolation` is never constructed anywhere in `src/`.

Source-verified: `ReduceOnly` / `set_reduce_only` / `ReduceOnlyViolation` appear only in
`state.rs`, `refusal.rs` (definition + `Display` + `rule()`), and the `set_reduce_only`
setter in `engine.rs`; no `match` arm and no `RiskRefusal::ReduceOnlyViolation { .. }`
construction exists in any evaluation path.

## Why

Reduce-only is a safety-critical mode: after a soft risk breach an operator should be able to
switch the platform to "close/reduce only, no new risk" without a full halt. Today the switch
is cosmetic — order placement continues to be approved under `ReduceOnly`, so the protection
it promises does not exist. An operator who triggers `set_reduce_only()` gets a false sense of
enforcement.

## How to close it

1. In `evaluate_core_checks` and `check_order_fast_checks` (step 1, before the OTR recording
   happens) reject order-*opening* orders under `GlobalState::ReduceOnly` with
   `RiskRefusal::ReduceOnlyViolation`. Decide the definition of "reduces risk" — e.g. allow
   `Sell` to close a long / `Buy` to cover a short, refuse new position entry and
   size-increasing modifies — or start strict (refuse all non-reducing orders) and refine.
2. Decide the same for `check_decision` (`PlaceOrder` path) and `RiskEngine::validate_sizing_request`.
3. Add tests in `tests/` (Red → Green): reduce-only set → new order refused with
   `reduce_only_violation`; a closing/reducing order approved; `resume_global()` restores
   full approval.

Related: collected under `docs/plans/0002-close-declared-unenforced-risk-controls.md`.
