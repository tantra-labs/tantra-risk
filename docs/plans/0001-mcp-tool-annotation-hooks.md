# 0001 — Tool-Call Annotation Hooks for the MCP Gateway (readOnlyHint / destructiveHint)

> **Status**: Proposed — not yet implemented.
> **Target Crates**: vyapaar-risk (annotation metadata only — the gateway itself is vyapaar-agent-mcp, out of scope here)
> **Depends on reading**:
> - vyapaar-agent/docs/plans/0001-mcp-gateway-discovery-and-quantitative-engines.md

---

## 0. Executive Summary & Objective

Fincept's MCP tools carry `readOnlyHint` / `destructiveHint` / `idempotentHint` annotations so a calling agent or host knows what needs human confirmation before the tool is even invoked. This is the agent-facing analogue of what `vyapaar-risk`'s synchronous pre-trade gate already does deterministically at execution time. `vyapaar-risk` needs no new engine for this — its existing `src/engine.rs`/`src/refusal.rs`/`src/limits.rs` pre-trade checks are already stronger than an advisory hint — but it should expose a small, pure classification function so the annotation logic lives next to the actual risk classification instead of being hand-duplicated inside the MCP crate.

---

## 1. Context

`vyapaar-agent/docs/plans/0001-mcp-gateway-discovery-and-quantitative-engines.md` already defines the MCP gateway's tool-discovery tiers and the `vyapaar/result/v1` envelope in `vyapaar-agent-core::result`. That doc does not specify where the destructive/read-only classification for order-affecting tools comes from. Since `vyapaar-risk` (`src/limits.rs`, `src/otr.rs`, `src/sizing_gate.rs`, `src/algo_registry.rs`, `src/engine.rs`, `src/refusal.rs`, `src/metrics.rs`, `src/state.rs`) is already the sub-15µs synchronous authority on what a given decision is allowed to do, it is the natural, single source of truth for *what kind* of action a decision represents, too.

---

## 2. Component Specifications

Add to `vyapaar-risk`:
- `ToolRiskHint` enum — `ReadOnly`, `Destructive`, `Idempotent` (mirroring the MCP annotation vocabulary: `readOnlyHint`, `destructiveHint`, `idempotentHint`), kept intentionally minimal and additive.
- `fn risk_classification(decision_kind: &DecisionKind) -> ToolRiskHint` — a pure function with no side effects, consuming the existing `Decision`/`DecisionKind` type from `vyapaar-core` (per `vyapaar-core/src/decision.rs`). Mapping: `Decision::Hold` → `ToolRiskHint::ReadOnly`; `Decision::PlaceOrder` / `Decision::ModifyOrder` / `Decision::CancelOrder` / `Decision::CancelAll` → `ToolRiskHint::Destructive`.

This function is consumed later by `vyapaar-agent-mcp` when it annotates its own tool schemas — not implemented here. `vyapaar-risk` itself gains only the enum and the pure mapping function; no change to the existing pre-trade gate, limits, or refusal logic.

---

## 3. Implementation Phases

### Phase 1: Define hint enum + classification function
Add `ToolRiskHint` and `risk_classification()` to `vyapaar-risk`, unit-test the mapping exhaustively against every `DecisionKind` variant, and leave consumption by `vyapaar-agent-mcp`'s tool-schema annotation as a follow-on change in that crate.
