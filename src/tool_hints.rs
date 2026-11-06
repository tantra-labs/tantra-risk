//! Tool-call annotation hooks for the MCP gateway (`readOnlyHint` /
//! `destructiveHint` / `idempotentHint`).
//!
//! Fincept's MCP tools carry these annotations so a calling agent or host
//! knows what needs human confirmation before a tool is even invoked. The
//! synchronous pre-trade gate in [`crate::engine`] is the deterministic,
//! execution-time authority on what a decision may do; this module is the
//! advisory, schema-time classification of *what kind of action* a decision
//! represents, so the annotation logic lives next to the risk classification
//! instead of being hand-duplicated inside the MCP crate.
//!
//! Pure and side-effect free: classification is a total, deterministic map
//! from the closed [`vyapaar_core::Decision`] grammar to a [`ToolRiskHint`].
//!
//! NOTE: `vyapaar-agent/docs/plans/0001` names a `DecisionKind` type in
//! `vyapaar-core` as the classification input. That type does not exist in
//! the kernel today, so this module classifies the closed [`Decision`] enum
//! directly — an exact, exhaustive mapping over its five variants — and
//! avoids inventing a redundant kernel type.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use vyapaar_core::Decision;

/// MCP tool-risk annotation vocabulary (`readOnlyHint`, `destructiveHint`,
/// `idempotentHint`), kept intentionally minimal and additive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolRiskHint {
    /// The tool only reads state; it can never change the book. No human
    /// confirmation is warranted.
    ReadOnly,
    /// The tool places, modifies, or cancels orders — it can move money.
    /// A host should require explicit human confirmation before invocation.
    Destructive,
    /// The tool is safe to retry with the same inputs (no state change on
    /// repeat). Reserved for tools that map here in future phases.
    Idempotent,
}

impl ToolRiskHint {
    /// The MCP annotation key this hint is serialized as on a tool schema:
    /// `readOnlyHint`, `destructiveHint`, or `idempotentHint`.
    pub const fn mcp_key(self) -> &'static str {
        match self {
            Self::ReadOnly => "readOnlyHint",
            Self::Destructive => "destructiveHint",
            Self::Idempotent => "idempotentHint",
        }
    }
}

/// Classifies a [`Decision`] into the advisory [`ToolRiskHint`] the MCP
/// gateway should attach to the corresponding tool schema.
///
/// Mapping:
/// - [`Decision::Hold`] → [`ToolRiskHint::ReadOnly`]
/// - [`Decision::PlaceOrder`] / [`Decision::ModifyOrder`] /
///   [`Decision::CancelOrder`] / [`Decision::CancelAll`] →
///   [`ToolRiskHint::Destructive`]
///
/// Every current `Decision` variant is covered exhaustively; a new order
/// affecting variant added to the kernel without a match here is a compile
/// error.
pub fn risk_classification(decision: &Decision) -> ToolRiskHint {
    match decision {
        Decision::Hold { .. } => ToolRiskHint::ReadOnly,
        Decision::PlaceOrder { .. }
        | Decision::ModifyOrder { .. }
        | Decision::CancelOrder { .. }
        | Decision::CancelAll { .. } => ToolRiskHint::Destructive,
    }
}
