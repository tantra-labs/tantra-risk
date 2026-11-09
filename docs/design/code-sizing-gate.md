# `vyapaar-risk` — Sizing Gate Reference

Documents `vyapaar-risk/src/sizing_gate.rs`: dynamic risk-budget pre-trade gate
for position sizing requests.

## `validate_sizing_request`

Validates a `PositionSizingRequest` against risk limits before a position is
sized. Returns `SizingRiskRefusal` on violation.

## `SizingRiskRefusal`

Embedded refusal payload returned by `RiskRefusal` — fully diagnosable on the
hot path. Contains the specific sizing rule that was violated and the current
risk budget utilization.

## Zero-Floats Policy

All risk budget values, position sizes, and utilization ratios use
`rust_decimal::Decimal` — exact 128-bit arithmetic.

## Related

- `vyapaar-core/src/sizing.rs` — `PositionSizingRequest`, `PositionSizingResult`, `RiskBudget`, `SizingMethod`.
- `vyapaar-ledger` — `RiskBudget` for portfolio-level risk budgets.
- `vyapaar-risk/src/engine.rs` — `RiskEngine` calls `validate_sizing_request` for sizing decisions.
