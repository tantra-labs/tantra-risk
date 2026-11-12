# 0002 — Close declared-but-unenforced risk controls

**Status**: open

## Scope

Closes the four source-verified gaps in `tantra-risk` where a declared config knob or API
surface is not enforced by any check path:

| Gap | Severity | Title |
|-----|----------|-------|
| [0001](../gaps/0001-price-collar-unenforced.md) | Medium | Market-order price collar (`price_collar_bps`) declared but never enforced |
| [0002](../gaps/0002-reduce-only-not-enforced.md) | High | Reduce-only mode (`GlobalState::ReduceOnly`) not enforced by any check path |
| [0003](../gaps/0003-check-order-fast-otr-divergence.md) | Medium | `check_order_fast` skips per-scrip OTR check, warning zone, and cooldown auto-resume |
| [0004](../gaps/0004-otr-window-config-dead.md) | Low | OTR window config knobs (`window_1min_ms` / `window_5min_ms`) never consulted |

## Why

All four are behavioral: configuration or API exists and is validated/serialized, but no
evaluation path enforces it, so operators can set controls that silently do nothing and the
fast path can disagree with the full path on scrip OTR enforcement. Closing them restores
the guarantee that every declared limit is a real limit.

## Phases

1. **0002 (High)** — enforce reduce-only: reject non-reducing orders under
   `GlobalState::ReduceOnly` on both `evaluate_core_checks` and `check_order_fast_checks`;
   construct `RiskRefusal::ReduceOnlyViolation`; cover `check_decision` and
   `validate_sizing_request`.
2. **0001 (Medium)** — apply the `price_collar_bps` market-order deviation collar around the
   band reference price.
3. **0003 (Medium)** — bring `check_order_fast` per-scrip OTR to parity (record-and-check the
   scrip tracker before approval); decide/keep the warning-zone and cooldown behaviors with
   tests pinning the guarantee.
4. **0004 (Low)** — wire the configured OTR windows into the tracker / enforcement, or remove
   the dead knobs.

## Verification

TDD per phase: write failing tests in `tests/` first (one per refusal/behavior), then the
minimal `src/` change. Finish with:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

When a gap is verified closed, remove its doc from `docs/gaps/`, update `docs/gaps/README.md`
(move the row to Closed with the closing commit), and remove this plan once all four are done.
