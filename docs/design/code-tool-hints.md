# `tantra-risk` — Tool Risk Hints Reference

Documents `tantra-risk/src/tool_hints.rs`: MCP tool-call annotation hooks. Fincept's MCP
tools carry `readOnlyHint` / `destructiveHint` / `idempotentHint` annotations so a calling
agent or host knows what needs human confirmation before a tool is even invoked. The
synchronous pre-trade gate in `engine.rs` is the execution-time authority on what a decision
may do; this module is the advisory, schema-time classification of *what kind of action* a
decision represents.

## `ToolRiskHint`

| Variant | `mcp_key()` | Meaning |
|---------|-------------|---------|
| `ReadOnly` | `"readOnlyHint"` | Reads state only; never changes the book. No human confirmation warranted. |
| `Destructive` | `"destructiveHint"` | Places/modifies/cancels orders — can move money. Host should require explicit confirmation. |
| `Idempotent` | `"idempotentHint"` | Safe to retry with the same inputs (no state change on repeat); reserved for future tools. |

There are exactly **three** variants — there is no `Emergency` hint.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolRiskHint { ReadOnly, Destructive, Idempotent }

impl ToolRiskHint {
    pub const fn mcp_key(self) -> &'static str { /* readOnlyHint | destructiveHint | idempotentHint */ }
}
```

## `risk_classification`

```rust
pub fn risk_classification(decision: &Decision) -> ToolRiskHint
```

A pure, total, deterministic map over the closed `tantra_core::Decision` grammar:

| Decision variant | Hint |
|------------------|------|
| `Hold` | `ReadOnly` |
| `PlaceOrder` | `Destructive` |
| `ModifyOrder` | `Destructive` |
| `CancelOrder` | `Destructive` |
| `CancelAll` | `Destructive` |

Every current `Decision` variant is covered exhaustively — a new order-affecting variant
added to the kernel without a match here is a compile error. Classification is based purely
on the decision variant (not tool name, parameters, or system state).

## Notes

- `tantra-agent/docs/plans/0001` names a `DecisionKind` type in `tantra-core` as the
  classification input. That type does not exist in the kernel today, so this module
  classifies the closed `Decision` enum directly and avoids inventing a redundant kernel type.

## Related

- Plan 0001 — Tool-Call Annotation Hooks for the MCP Gateway, commit `5b00cda`
  (`ToolRiskHint`, `risk_classification`, `tests/tool_hints_tests.rs`).
- `tantra-risk/tests/tool_hints_tests.rs` — exhaustive variant + `mcp_key()` + serde tests.
