# `vyapaar-risk` — Algo ID Registry Reference

Documents `vyapaar-risk/src/algo_registry.rs`: SEBI Retail Algo Circular registry
tracking approved `AlgoId`s and per-algo state.

## `AlgoIdRegistry`

Maintains the registry of SEBI-assigned algorithmic identifiers. Every order must
carry a valid, registered `algo_id`. Per-algo state tracks:

| State | Meaning |
|-------|---------|
| `Active` | Normal trading permitted |
| `ReduceOnly` | Only reduce-existing-position orders permitted |
| `Halted` | All trading suspended for this algo |

## Key Responsibilities

- Validate `algo_id` format against `^[A-Za-z0-9_-]{4,32}$`
- Track per-algo operational state transitions
- Maintain per-algo OTR trackers and cumulative turnover/PnL
- Support emergency per-algo halt and resume

## Zero-Floats Policy

No financial math — all values are identifiers, states, or counters.

## Related

- `vyapaar-risk/src/engine.rs` — `RiskEngine` consults `algo_registry` on every check.
- `vyapaar-risk/src/state.rs` — `AlgoState` type definition.
- `vyapaar-core/src/decision.rs` — `Decision` enum carries `algo_id` field.
- `vyapaar-exec/src/executor.rs` — `RiskGatedExecutor` enforces algo-level risk.
