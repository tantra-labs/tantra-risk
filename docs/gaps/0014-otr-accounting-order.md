# 0014 — Side effects recorded before later checks reject: refused orders inflate OTR and rate counters; scrip OTR breach halts the whole algo

**Status**: open
**Filed**: 2026-11-23 — workspace design review (root plan 0021) · **Kind**: design-flaw · **Severity**: medium
**Plan**: [0003](../plans/0003-design-review-2026-11.md)

## What's stubbed or hardcoded

- `src/engine.rs:~598-604` (`record_order_placement` before the OTR check) and `~632-640` (scrip tracker) run before the price band, quantity, notional and daily-loss checks (steps 4-8). If step 4 or later refuses, the placement stays counted although no order reaches the exchange.
- `src/engine.rs:~655`: `halt_algo(algo_id, "Scrip OTR breached for ..")` halts the entire algo for a per-scrip breach.
- Both check pipelines duplicate this logic (`check_order_fast_checks`); merging them is root plan 0019 P3.2.
- `src/engine.rs:~1189-1197` `suggest_freeze_slices` uses `(q + f - 1) / f` on `Decimal` then `to_i32()` (truncation): for q = 1800.5, f = 1800 it returns 1, not 2. The integer ceil trick is invalid for `Decimal`.
- `src/engine.rs:~1270` `let _ = now;` keeps an unused parameter in `check_trade_plan`; `check_position_concentration` and `check_mis_leverage` (`~1171-1183`, `~1215`) do not call `record_refusal`, so they are missing from `refusal_counts`.
- Slicing is implemented independently in `tantra-exec/crates/tantra-exec-slicer/src/freeze.rs`.

## Why

A strategy that keeps hitting a freeze-limit refusal drives its own OTR (denominator 1, no fills) above 50 and is auto-halted by the gate, and one illiquid scrip halts every instrument of that algo. SEBI OTR counts orders sent to the exchange, not locally refused ones. Fractional quantities are under-sliced, risking exchange freeze rejection, and refusal accounting is inconsistent.

## How to close it

1. Evaluate all pure checks first and commit state mutations only on approval (`Verdict` plus `commit`), applying one ordering to both pipelines.
2. Add a per-scrip halt state so a scrip breach no longer halts the algo; document exactly what counts toward OTR.
3. Route all refusals through a single `refuse()` helper that records the refusal uniformly.
4. Replace the slice-count arithmetic with `(q / f).ceil()`, or delete `suggest_freeze_slices` in favour of the exec slicer catalog.
5. Drop or use the `now` parameter in `check_trade_plan`.
6. Deferring the commit must not add allocation or a second pass; keep the sub-15 µs budget with a benchmark.

**Verification**: proptest that a refused order leaves OTR/rate counters byte-equal to their prior state; `tests/otr_accounting_tests.rs` (repeated freeze refusals do not halt the algo; scrip breach halts only that scrip); proptest slice count >= q/f for fractional q; `cargo test -p tantra-risk --profile prod`, `cargo bench`.

## Related

- Root plan 0019 P3.2 (merge pipelines) - this gap is the behavioural fix.
- [0008](0008-bounded-rate-windows.md), [0009](0009-trading-day-rollover.md); plan R4 (duplicated freeze catalog and OTR definitions across repos).
