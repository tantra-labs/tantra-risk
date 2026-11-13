# Gap 0005 — No Stop-Protection (SL / TSL) Risk Validation

**Status**: open — tracked by root master plan [`docs/plans/0005-elephant-edge-strategy-platform.md`](../../../docs/plans/0005-elephant-edge-strategy-platform.md)

## What's Missing

`tantra-risk` enforces reduce-only, the market-order price collar
(`price_collar_bps`), OTR windows, circuit bands, and freeze limits — but it
has **no stop-protection checks**. The Elephant Edge (EE+) strategy
(`tantra-strategies` plan 0001) attaches a `StopProtection` (initial SL + ATR /
swing TSL) to every signal; the risk gate cannot currently verify, at
sub-15 µs:

1. **Mandatory SL**: intraday opening orders without a stop are refused
   (SEBI-style discipline — no unbounded intraday risk).
2. **SL-distance band**: `|entry − sl|` must be within `[min_sl_atr × ATR,
   max_sl_pct × reference]` — too tight (costs/liquidity) or too wide
   (excessive risk) is refused, mirroring the existing `price_collar_bps`
   band pattern.
3. **SL within the price collar**: the stop may not sit outside the reference
   ± collar band already enforced for the order.
4. **Max-loss-per-strategy**: a `StopProtection` whose worst-case loss ×
   position would breach the strategy's `max_loss` risk budget is refused
   (ties into `tantra-strategies` gap 0005 `RiskBudget`).
5. **TSL legitimacy**: trailing-stop `modify_stop` requests (from
   `tantra-exec` gap 0003) must tighten — never widen — a stop beyond the
   original SL (prevents stop-widening abuse on the modify path).

## Why

The NoFX invariant (`.agents/AGENTS.md` §5.5) treats AI/strategy outputs as
untrusted and clamps them at the gate. An intraday system whose stops are
attached only in the strategy layer is a stop-optional system: the gate must
independently verify the stop math and the modify path so no order — human,
strategy, or AI-generated — reaches execution without a valid stop or with an
invalidated trailing stop. This closes the loop that `tantra-exec` gap 0003
(execution) and `tantra-ledger` gap 0012 (OMS lifecycle) depend on.

## How to Close It

1. Extend the order/decision check inputs with the attached
   `StopProtection` (from `tantra-core` plan 0003 types) and the scrip ATR +
   strategy `max_loss` context.
2. New `RiskRefusal` variants (mirroring `PriceCollarViolation`):
   `MissingStopProtection`, `StopTooTight`, `StopTooWide`,
   `StopOutsidePriceCollar`, `MaxLossExceeded`, `StopWideningDenied`.
3. Enforce on **every** check path (`evaluate_core_checks` +
   `check_order_fast_checks`) plus the bracket/modify path (`validate_modify`),
   keeping the synchronous < 15 µs latency invariant (pure Decimal compares,
   no I/O).
4. Tests in `tests/` (TDD): each new refusal, boundary proptests, and the
   no-widening TSL rule; golden vectors for the band arithmetic.

## Verification

`cargo test --workspace --profile prod` green in `tantra-risk`; every opening
order without a valid `StopProtection` is refused; a TSL `modify_stop` that
widens the stop is refused; `make prod` green end-of-turn (build-discipline
rule).
