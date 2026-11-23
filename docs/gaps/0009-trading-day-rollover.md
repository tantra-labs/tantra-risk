# 0009 — No trading-day rollover: daily loss, turnover, PnL and OTR counters never reset; daily-loss halt is global and sticky

**Status**: open
**Filed**: 2026-11-23 — workspace design review (root plan 0021) · **Kind**: design-flaw · **Severity**: high
**Plan**: [0003](../plans/0003-design-review-2026-11.md)

## What's stubbed or hardcoded

- `src/engine.rs:36-60`: `daily_loss`, `cumulative_turnover`, `cumulative_pnl` and `algo_trackers` have no reset API (no `fn reset`, `new_session` or `trading_day` exists in `src/`).
- `src/engine.rs:~735-750`: a breach of `max_daily_loss` calls `halt_global`; `daily_loss` is one engine-wide `Decimal`, not per-algo or per-account.
- `src/otr.rs:~89-130`: cumulative `orders_placed` / `trades_executed` feed `is_breached` / `warning_ratio` for the whole process life, while enforcement uses the one-minute window, so there are two OTR definitions.
- `orders_modified` is never incremented anywhere; `record_order_mod_or_cancel` bumps `orders_cancelled`.

## Why

The engine carries yesterday's loss into today (immediate global halt at open) or, if recreated each day, loses halts on restart (see [0010](0010-shared-risk-state-and-kill-switch.md)). One strategy's loss halts every strategy and account. The trading day for Indian markets is IST, not UTC; a UTC boundary would roll over mid-session.

## How to close it

1. Introduce a `TradingDay` value (IST date, derived from the `tantra-core` calendar) supplied by the caller alongside `now`.
2. Add `roll_session(day)` that clears day-scoped state (loss, turnover, PnL, OTR counters, order history) and is idempotent for the same day.
3. Make `daily_loss` and its limit per-algo (and per-account where available); keep a global aggregate as a separate, explicit limit.
4. Split `GlobalState::Halted` causes (operator vs limit breach) so auto-resume rules differ.
5. Unify the cumulative and windowed OTR definitions and fix `orders_modified` accounting.
6. Rollover must be O(1) or off the hot path so the sub-15 µs check path is unaffected.

**Verification**: `tests/session_rollover_tests.rs` (loss carried across a day boundary is cleared; IST midnight boundary vs UTC; idempotent roll; per-algo isolation of a loss halt); proptest that state after `roll_session` equals fresh state for day-scoped fields; `cargo test -p tantra-risk --profile prod`.

## Related

- Root gap 0020 (time semantics: IST trading day vs UTC, missing day rollover); tantra-core calendar session model.
- [0008](0008-bounded-rate-windows.md) (reset hook), [0011](0011-encapsulated-state-and-operator-audit.md) (halt cause and resume authorisation).
