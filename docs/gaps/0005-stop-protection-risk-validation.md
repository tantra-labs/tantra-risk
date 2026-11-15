# Gap 0005 — No Stop-Protection (SL / TSL) & Weight/ADV Risk Validation

**Status**: open — tracked by root master plans [`docs/plans/0005-elephant-edge-strategy-platform.md`](../../../docs/plans/0005-elephant-edge-strategy-platform.md) and [`docs/plans/0008-trade-signal-generator.md`](../../../docs/plans/0008-trade-signal-generator.md)

## What's Missing

`tantra-risk` enforces reduce-only, the market-order price collar
(`price_collar_bps`), OTR windows, circuit bands, freeze limits, and basic
notional/qty sizing checks (`src/sizing_gate.rs`) — but it has **no stop-protection
checks** and **no weight / ADV participation limits**:

1. **Mandatory SL**: intraday opening orders without a stop are refused
   (SEBI-style discipline — no unbounded intraday risk).
2. **SL-distance band**: `|entry − sl|` must be within `[min_sl_atr × ATR,
   max_sl_pct × reference]` (or `<= max_stop_adr_mult × ADR%`) — too tight
   (costs/liquidity) or too wide (excessive risk) is refused, mirroring the
   existing `price_collar_bps` band pattern.
3. **SL within the price collar**: the stop may not sit outside the reference
   ± collar band already enforced for the order.
4. **Max-loss-per-strategy**: a `TradePlan` whose worst-case loss × position
   (including post-fill slippage re-anchoring) would breach the strategy's
   `max_loss` risk budget is refused.
5. **TSL legitimacy**: trailing-stop `TradeAction::MoveStop` / `modify_stop`
   requests must tighten — never widen — a stop beyond the current or
   original SL (prevents stop-widening abuse on the modify path).
6. **Weight & ADV20 caps in `sizing_gate.rs`**: `RiskBudget` / `PositionSizingRequest`
   and `SizingRiskRefusal` lack `max_weight_pct` and `max_adv_pct` over `adv20`,
   and `SizingRiskRefusal::fmt` currently hardcodes `"INR"` instead of formatting
   `Money.currency` (breaking multi-market `qullamaggie` US/India plans).

## Why

The NoFX invariant (`.agents/AGENTS.md` §5.5) treats AI/strategy outputs as
untrusted and clamps them at the gate. A trade plan or LLM-generated override
whose stop, weight, or ADV limit is checked only in the strategy/planner layer
is stop-optional at the execution boundary: the gate must independently verify
`TradePlan` stop math, sizing caps, and the modify path at sub-15 µs latency so
no order — human, strategy, or AI-generated — reaches execution without a valid,
risk-checked stop. This closes the loop that `tantra-exec` gap 0003 (execution)
and `tantra-ledger` gap 0012 (OMS lifecycle) depend on.

## How to Close It

1. Extend the order/decision/sizing check inputs to consume **`TradePlan` /
   `TradePlanSpec`** stop & trail contracts from `tantra-core` plan `0005`
   (unified with `0003` — no duplicate `StopProtection` struct), plus scrip ATR /
   ADR%, `adv20`, and strategy `max_loss`.
2. New `RiskRefusal` / `SizingRiskRefusal` variants:
   `MissingStopProtection`, `StopTooTight`, `StopTooWide`,
   `StopOutsidePriceCollar`, `MaxLossExceeded`, `StopWideningDenied`,
   `MaxWeightExceeded`, `MaxAdvParticipationExceeded`; replace hardcoded `"INR"`
   in `SizingRiskRefusal::fmt` with `equity.currency` / `risk_at_stake.currency`.
3. Enforce on **every** check path (`evaluate_core_checks`,
   `check_order_fast_checks`, `validate_sizing_request`) plus the bracket/modify
   path (`validate_modify`), keeping the synchronous < 15 µs latency invariant
   (pure `Decimal` compares, no I/O).
4. Tests in `tests/` (TDD): each new refusal, multi-currency refusal formatting,
   boundary proptests, and the no-widening TSL rule; golden vectors for band arithmetic.

## Verification

`cargo test --workspace --profile prod` green in `tantra-risk`; every opening
order without a valid `TradePlan` stop (or breaching weight/ADV caps) is
refused; any TSL `modify_stop` that widens the stop is refused; `make prod`
green end-of-turn (build-discipline rule).
