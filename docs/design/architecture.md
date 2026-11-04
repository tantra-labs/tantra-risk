# `vyapaar-risk` — DDD Architecture & Pre-Trade Regulatory Gate

## 1. Bounded Context & Responsibility
`vyapaar-risk` (Tier 1A, ported from `honba-mon`) is the **authoritative, synchronous pre-trade regulatory risk gate** for Indian equities and derivatives (NSE/BSE/NFO/MCX). Every order placement, modification, cancellation, and position-sizing request must pass through `RiskEngine` before reaching `vyapaar-exec`.

## 2. Performance & Purity SLOs
* **Latency**: $p99.99 < 15\,\mu\text{s}$ ($p50 < 1\,\mu\text{s}$) via zero-allocation `RiskEngine::check_order_fast`.
* **Zero-I/O & Temporal Determinism**: 100% synchronous in-memory evaluation; all methods accept explicit `DateTime<Utc>`.
* **Zero-Floats**: All notional, price-band, loss, and OTR calculations use `rust_decimal::Decimal`.

## 3. Domain Aggregates & Services (`src/`)
* `RiskEngine` (`src/engine.rs`): Root aggregate coordinating all pre-trade checks.
* `AlgoIdRegistry` (`src/algo_registry.rs`): SEBI Retail Algo Circular registry tracking approved `AlgoId`s and per-algo state (`Active`, `ReduceOnly`, `Halted`).
* `OtrTracker` (`src/otr.rs`): Sliding-window Order-to-Trade Ratio calculator enforcing NSE/SEBI OTR tiers.
* `RiskLimits` & `OrderRateLimit` (`src/limits.rs`): Max order notional, max position quantity, freeze quantity limit, daily loss limit, and price band checks.
* `validate_sizing_request` (`src/sizing_gate.rs`): Dynamic risk-budget pre-trade gate.
* `RiskMetrics` (`src/metrics.rs`): Lock-free atomic latency histograms and breach counters.
