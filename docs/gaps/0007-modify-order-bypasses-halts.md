# 0007 — `Decision::ModifyOrder` bypasses global halt, algo halt, registry and market-circuit checks

**Status**: open
**Filed**: 2026-11-23 — workspace design review (root plan 0021) · **Kind**: design-flaw · **Severity**: high
**Plan**: [0003](../plans/0003-design-review-2026-11.md)

## What's stubbed or hardcoded

- `src/engine.rs:~395-415` (`check_decision`, `Decision::ModifyOrder` arm): only `GlobalState::ReduceOnly` is tested, then `record_cancellation` is called and the arm returns `Ok(())`.
- The arm performs no `GlobalState::Halted` check, no `algo_states` check, no `algo_registry` check and no `market_circuit` check.
- The `instrument` field is discarded (`let _ = instrument`), so price band, freeze limit and notional are not re-checked on a modify that can raise price or quantity.

## Why

During a platform kill-switch or after an OTR auto-halt, an AI `Decision::ModifyOrder` is still approved and can resize or reprice a resting order past band and freeze limits. This contradicts the rule that every AI `Decision` passes the gate unconditionally (`.agents/rules/coding-and-ddd.md` section 1.4, Untrusted AI Contract). A modify is also counted as a cancellation in OTR accounting, which hides modify churn.

## How to close it

1. Refuse `ModifyOrder` under `GlobalState::Halted`, when the algo is halted, and when the algo is unregistered; honour market-circuit state.
2. Require the new price and quantity on the modify decision (tantra-core `Decision` shape, see [0012](0012-decision-exchange-and-algo-id-typing.md)) and run band / freeze / notional / collar on them, or refuse modifies that increase exposure.
3. Count modifies as modifies (`orders_modified` is never incremented today; see [0009](0009-trading-day-rollover.md)).
4. Keep cancels allowed under halt and document that explicitly.
5. Reuse the same check helpers as the placement path so the added checks are flag/compare only and stay inside the sub-15 µs budget.

**Verification**: `tests/modify_order_tests.rs` with one negative test per state (Halted, algo-halted, unregistered, market circuit, band breach, freeze breach, exposure increase) and a positive test for a reducing modify; proptest that Modify under Halted is always refused; `cargo clippy --all-targets -- -D warnings`, `cargo test -p tantra-risk --profile prod`.

## Related

- Root gap 0013 (single, fail-closed gate).
- [0006](0006-fail-closed-reference-data.md) (modify re-check needs reference data to fail closed); [0011](0011-encapsulated-state-and-operator-audit.md).
- tantra-core `Decision::ModifyOrder` shape (see tantra-core plan 0006 R5 for id typing).
