# `vyapaar-risk` — Refusal & State Reference

Documents `vyapaar-risk/src/refusal.rs` and `vyapaar-risk/src/state.rs`.

## `RiskRefusal` (`refusal.rs`)

The refusal type returned when a decision or order fails pre-trade risk checks.
Deliberately returns large aggregate payloads by value so the refusal reason is
fully diagnosable on the hot path. `clippy::result_large_err` is consciously
accepted at the crate level.

## `AlgoState` / `GlobalState` / `MarketCircuitHalt` (`state.rs`)

| Type | Variants | Purpose |
|------|----------|---------|
| `GlobalState` | `Active`, `ReduceOnly`, `Halted` | Platform-wide trading state |
| `AlgoState` | `Active`, `ReduceOnly`, `Halted` | Per-algo operational state |
| `MarketCircuitHalt` | index-level circuit breaker | Market-wide halt (10%/15%/20% bands) |

## State Transitions

- `Active` → `ReduceOnly`: Soft risk breach, reduce existing positions only
- `ReduceOnly` → `Halted`: Hard risk breach, no new orders
- `Halted` → `Active`: Manual resume (or auto-resume after `halt_cooldown_ms`)
- Any state → `MarketCircuitHalt`: Exchange-level circuit band breach

## Zero-I/O & Zero-Floats

All state types are plain data — no async, no I/O, no floats. Transitions are
synchronous and deterministic.

## Related

- `vyapaar-risk/src/engine.rs` — `RiskEngine` holds `global_state`, `algo_states`, `market_circuit`.
- `vyapaar-risk/src/algo_registry.rs` — `AlgoIdRegistry` tracks per-algo state.
