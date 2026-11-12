# `vyapaar-risk` — Sizing Gate Reference

Documents `vyapaar-risk/src/sizing_gate.rs` and the `RiskEngine::validate_sizing_request`
entry point: the dynamic risk-budget pre-trade gate for position sizing requests. It
synchronously validates a `vyapaar_core::PositionSizingRequest` against a
`vyapaar_core::RiskBudget` before sizing decisions can emit executable orders.

## `SizingRiskRefusal`

Typed refusal reason from the sizing gate (returned inside `RiskError::SizingRefusal`),
with a stable `rule()` name per variant:

| Variant | `rule()` | Meaning |
|---------|----------|---------|
| `PortfolioRiskCapExceeded` | `portfolio_risk_cap_exceeded` | Requested/computed risk % > `max_portfolio_risk_pct` |
| `NonLotMultiple` | `non_lot_multiple` | Sized quantity not a multiple of the lot size |
| `BelowMinimumLotSize` | `below_minimum_lot_size` | Sized quantity is zero or below minimum lot |
| `FreezeLimitBreached` | `freeze_limit_breached` | Sized quantity > exchange freeze limit |
| `MaxQuantityExceeded` | `max_quantity_exceeded` | Sized quantity > `max_position_qty` |
| `MaxNotionalExceeded` | `max_notional_exceeded` | Sized notional > `max_position_notional` |
| `InvalidEquity` | `invalid_equity` | Account equity zero/negative |
| `InvalidEntryPrice` | `invalid_entry_price` | Entry price not strictly positive |
| `InvalidStopLoss` | `invalid_stop_loss` | Stop loss inconsistent with entry price / `PositionSide` |
| `PlatformHalted` | `platform_halted` | `GlobalState::Halted` (from the engine entry point) |
| `CalculationError(String)` | `calculation_error` | Underlying `calculate_position_size` failure |

## `sizing_gate::validate_sizing_request`

```rust
pub fn validate_sizing_request(
    request: &PositionSizingRequest,
    budget: &RiskBudget,
) -> Result<(), RiskError>
```

Enforcement steps (in source order):

1. **Entry price sanity** — must be strictly positive (`InvalidEntryPrice`).
2. **Equity sanity** — `min(budget.equity, request.account_equity)` must be `> 0`
   (`InvalidEquity`).
3. **Stop-loss / side consistency** — a positive stop loss must be below the entry price
   for `PositionSide::Long` and above it for `Short` (`InvalidStopLoss`).
4. **Method-level risk cap** — `SizingMethod::FixedFractional { risk_pct }` with
   `risk_pct > max_portfolio_risk_pct` is rejected up front (`PortfolioRiskCapExceeded`).
5. **Deterministic sizing** — delegates to `vyapaar_core::calculate_position_size` with the
   engine-supplied budget aligned into the request; failures become `CalculationError`.
6. **Actual risk-at-stake cap** — the computed `risk_at_stake / equity` must not exceed
   `max_portfolio_risk_pct` (`PortfolioRiskCapExceeded`).
7. **Lot size conformity** — zero quantity → `BelowMinimumLotSize`; non-multiple of
   `min_lot_size` → `NonLotMultiple`.
8. **Exchange freeze limit** — sized quantity ≤ `freeze_limit_qty` (`FreezeLimitBreached`).
9. **Max quantity cap** — ≤ `max_position_qty` (`MaxQuantityExceeded`).
10. **Max notional cap** — `allocated_notional ≤ max_position_notional`
    (`MaxNotionalExceeded`).

## `RiskEngine::validate_sizing_request`

```rust
pub fn validate_sizing_request(
    &self,
    request: &PositionSizingRequest,
    budget: &RiskBudget,
) -> Result<(), RiskError>
```

Rejects immediately on `GlobalState::Halted` (`SizingRiskRefusal::PlatformHalted`), then
backfills `effective_budget.freeze_limit_qty` from `self.limits.freeze_limits` when the
budget omits it, and finally delegates to `sizing_gate::validate_sizing_request`. Budget
fields are taken from the supplied budget, falling back to the request's own `risk_budget`
where the budget omits them.

## Zero-Floats Policy

All risk budget values, position sizes, and utilization ratios use `rust_decimal::Decimal`
and `vyapaar_core` `Money` / `Price` / `Quantity` wrappers — exact arithmetic, no floats.

## Zero-I/O / Temporal Determinism

Pure synchronous validation, no I/O. The only clock interaction is the engine entry point
checking `GlobalState::Halted` (a state already timestamped at halt time); no wall-clock
queries occur here.

## Related

- `vyapaar-core/src/sizing.rs` — `PositionSizingRequest`, `PositionSizingResult`, `RiskBudget`,
  `SizingMethod`, `calculate_position_size`.
- `vyapaar-risk/src/error.rs` — `RiskError::SizingRefusal(#[from] SizingRiskRefusal)`.
- `vyapaar-risk/src/engine.rs` — `RiskEngine::validate_sizing_request`.
