# 0011 — All `RiskEngine` state is `pub` and mutable; resume has no authorisation or audit record

**Status**: open
**Filed**: 2026-11-23 — workspace design review (root plan 0021) · **Kind**: design-flaw · **Severity**: medium
**Plan**: [0003](../plans/0003-design-review-2026-11.md)

## What's stubbed or hardcoded

- `src/engine.rs:~32-60`: every field (`global_state`, `algo_states`, `limits`, `price_bands`, `refusal_counts`, `order_history`, `daily_loss`) is `pub`.
- `resume_global(&mut self)` (`src/engine.rs:~124`) and `resume_algo` take no actor identity and emit only a `tracing::info!`.
- `src/algo_registry.rs:~38-62`: `register` / `reactivate` / `unregister` are likewise unauthenticated.
- Only `halt_*` carry a timestamp; `resume_*` cannot be audited with a time.
- `src/engine.rs:~237` `record_refusal` only increments `refusal_counts: HashMap<String, u64>`; the refusal reason is not stored, returned to a sink or timestamped, and hot-path refusals do not log the algo id or instrument.
- The mapping from `RiskRefusal` to a structured breach record is done outside the gate and only on some paths (REST `orders.rs` `check_risk`, `RiskActor`; `tantra-runtime-persistence/src/messages.rs:12`). Callers of `check_order_fast` or the FFI leave no audit record.

## Why

Any holder of `&mut RiskEngine` (including strategy-adjacent code such as `tantra-agent-mcp/src/tools.rs:1102` and the FFI Python `resume_global`) can clear a daily-loss kill switch or zero `daily_loss`. SEBI requires kill-switch actions and refusals to be auditable and retained; in-memory counters die with the process.

## How to close it

1. Make engine fields private with read accessors.
2. Change to `resume_global(operator: OperatorId, now) -> StateChange` and equivalents for algo resume and registry mutation; `StateChange` is an audit record (who, when, from, to, reason).
3. Split a privileged `RiskControl` trait from a check-only `RiskGate` trait so strategies are handed only `&dyn RiskGate`.
4. Require a stricter resume path for states auto-halted by a limit breach.
5. Emit a structured verdict record (`decision_id`, `algo_id`, `instrument`, `rule`, `inputs_digest`, `now`) and state-transition events to the injected observer (plan R1); the runtime appends them to the hash-chained audit WAL. Observer calls must be zero-cost with the no-op observer to protect the sub-15 µs path.

**Verification**: negative tests in `tests/` (strategy-facing handle cannot resume; resume without operator is a compile error); golden JSON vectors for verdict and `StateChange` records; `cargo clippy --all-targets -- -D warnings`, `cargo test -p tantra-risk --profile prod`.

## Related

- Root gap 0023 (tamper-evident audit trail unification); `tantra-exec/docs/gaps/0009-audit-wal-integrity.md`.
- [0007](0007-modify-order-bypasses-halts.md), [0010](0010-shared-risk-state-and-kill-switch.md).
