# `tantra-risk` — Algo ID Registry Reference

Documents `tantra-risk/src/algo_registry.rs`: the SEBI Retail Algo Circular registry of
exchange-registered algorithmic identifiers. Under the SEBI Retail Algo Circular (effective
August 1, 2025), every algorithmic order must carry a registered `algo_id`; the registry is
consulted synchronously during pre-trade risk evaluation.

## `AlgoRegistrationStatus`

Per-algo registration status (note: there is **no** `ReduceOnly` and **no** `Halted`
variant here — operational halt state lives on `AlgoState` in `state.rs`):

| Variant | Meaning |
|---------|---------|
| `Active` | Algo ID is registered and active; orders may be placed. |
| `Suspended { reason: String }` | Registered but suspended; orders are rejected. |
| `NotRegistered` | Not registered at all; any order is rejected. |

## `AlgoIdRegistry`

```rust
#[derive(Debug, Clone, Default)]
pub struct AlgoIdRegistry {
    registry: HashMap<String, AlgoRegistrationStatus>,
}
```

| Method | Signature | Purpose |
|--------|-----------|---------|
| `new` | `fn new() -> Self` | Create an empty registry. |
| `register` | `fn register(&mut self, algo_id: impl Into<String>)` | Insert as `Active`. |
| `suspend` | `fn suspend(&mut self, algo_id: &str, reason: impl Into<String>)` | Set `Suspended { reason }`. |
| `reactivate` | `fn reactivate(&mut self, algo_id: &str)` | Set back to `Active`. |
| `unregister` | `fn unregister(&mut self, algo_id: &str)` | Remove from the registry. |
| `is_registered_and_active` | `fn is_registered_and_active(&self, algo_id: &str) -> Result<(), String>` | `Ok(())` only for `Active`; descriptive error for `Suspended` / `NotRegistered` / unknown. |
| `len` | `fn len(&self) -> usize` | Number of registered algo IDs. |
| `is_empty` | `fn is_empty(&self) -> bool` | Registry is empty. |
| `registered_ids` | `fn registered_ids(&self) -> Vec<String>` | All registered algo IDs. |

There is **no** `algo_id` format regex in this module or elsewhere in the crate; validation
is purely against registry membership.

## Key Responsibilities

- Track per-algo registration status and state transitions (`register` / `suspend` /
  `reactivate` / `unregister`).
- Reject orders from unregistered or suspended algo IDs during pre-trade evaluation
  (`RiskEngine` consults `is_registered_and_active` on every check — see
  `code-engine.md`).

## Zero-Floats Policy

No financial math — all values are identifiers, states, or counters.

## Zero-I/O / Temporal Determinism

Pure in-memory `HashMap`; no I/O, no wall-clock queries. No timestamps are involved (halt
timing lives on `AlgoState::Halted` in `state.rs`).

## Related

- `tantra-risk/src/engine.rs` — `RiskEngine` consults `algo_registry` on every check
  (`evaluate_core_checks` step 2.5, `check_order_fast_checks` step 2.5).
- `tantra-risk/src/state.rs` — `AlgoState` (operational halt state) definition.
- `tantra-risk/src/refusal.rs` — `RiskRefusal::AlgoNotRegistered`.
- `tantra-core/src/decision.rs` — `Decision` carries the `algo_id` field.
