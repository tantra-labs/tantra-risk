# 0003 — `check_order_fast` skips per-scrip OTR check, warning zone, and cooldown auto-resume

**Status**: open

## What's stubbed or hardcoded

`check_order_fast_checks` (`src/engine.rs`) is the zero-allocation hot path. Compared with
`evaluate_core_checks` (the full path behind `check_order` / `check_decision`) it diverges on
three OTR behaviors:

1. **Per-scrip OTR circuit breaker is skipped on the hot path.** The full path records the
   scrip placement *then* checks `scrip_tracker.window_otr_1min(now)` against `max_otr` and
   halts on breach (`RiskRefusal::ScripOtrBreached`). The fast path only records the scrip
   placement **after all other checks pass** (`scrip_tracker.record_order_placement(now)` at
   the end of `check_order_fast_checks`) and never compares it against `max_otr` — so a scrip
   whose 1-min OTR is over the limit can be approved on the fast path.
2. **OTR warning zone is skipped** — the full path logs `tracker.warning_ratio(&self.otr_config)`
   when above `warning_threshold`; the fast path has no such logging.
3. **Halt cooldown auto-resume is skipped** — the full path auto-resumes an `AlgoState::Halted`
   algo once `now >= halted_at + halt_cooldown_ms`; the fast path rejects a halted algo
   unconditionally with `RiskRefusal::AlgoHalted`.

The divergence is documented in the `check_order_fast` doc comment and `docs/design/code-engine.md`.

Source-verified: `check_order_fast_checks` has no `window_otr_1min` call on the scrip
tracker, no `warning_ratio` call, and no `halt_cooldown_ms` logic; `evaluate_core_checks`
has all three.

## Why

`check_order_fast` and `check_order` are both public, live-safe entry points (fast path for
high-throughput algos, full path for everything else). Because they disagree on scrip OTR, a
scrip can be halted on one path while still being approved on the other, defeating the
per-scrip regulatory circuit breaker for fast-path users and making enforcement
path-dependent.

## How to close it

1. Bring the fast path to parity without sacrificing the zero-allocation invariant. The
   per-scrip tracker lookup can use `HashMap::get_mut` (already allocation-free — the current
   code does exactly that *after* approval); move the scrip OTR record-and-check before the
   approval decision and reject with `RiskRefusal::ScripOtrBreached` on breach, exactly like
   the full path.
2. Decide whether warning-zone logging and cooldown auto-resume are required on the fast path
   (they add no allocation — a `tracing` call and a timestamp compare) or whether they stay a
   documented full-path-only behavior. If kept divergent, state the guarantee explicitly and
   test that the scrip OTR circuit breaker **is** enforced on both paths.
3. Add tests in `tests/` (Red → Green): the same scrip OTR breach sequence must be refused by
   both `check_order` and `check_order_fast`; cooldown auto-resume behavior must be identical
   or explicitly documented as path-specific.

Related: collected under `docs/plans/0002-close-declared-unenforced-risk-controls.md`.
