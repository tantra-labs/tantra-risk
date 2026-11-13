# Reduce-Only Enforcement — Decision Record (`tantra-risk`)

**Status**: Accepted — plan 0002, gap 0002 (implemented 2026-11-13, commit `9a0bc59`)
**Scope**: `tantra-risk::RiskEngine` reduce-only semantics, price-collar semantics, and the
  position-free approximation they encode.

This is the permanent home of the reduce-only design decision; the per-repo HANDOFF and the
source comments only cite this document rather than restating it (`.agents/rules/handoff-hygiene.md`).

---

## 1. The decision

`GlobalState::ReduceOnly` ("existing positions can be closed/reduced, no new risk") is enforced
by **every** order check path in `RiskEngine`:

| Path | Behavior under `ReduceOnly` |
| :--- | :--- |
| `evaluate_core_checks` (full path, via `check_order` / `check_decision`) | `OrderSide::Buy` → `RiskRefusal::ReduceOnlyViolation`; `OrderSide::Sell` → approved |
| `check_order_fast_checks` (fast path) | identical to the full path |
| `check_decision` `ModifyOrder` | refused with `ReduceOnlyViolation` (risk impact cannot be verified without order state) |
| `check_decision` `CancelOrder` / `CancelAll` | approved (exposure-reducing) |
| `RiskEngine::validate_sizing_request` | refused with `SizingRiskRefusal::ReduceOnlyViolation` (sizing always establishes new exposure) |
| `resume_global` | restores full approval |

The refusal carries the machine-readable rule name `reduce_only_violation`
(`RiskRefusal::rule()` / `SizingRiskRefusal::rule()`).

## 2. Why the position-free approximation

`tantra-risk` is a **pure, zero-I/O, position-free** pre-trade gate: it has no position book,
no market data, and no wall clock. It therefore cannot verify whether a given order actually
reduces an existing position. Two options were considered:

1. **Side-based approximation (adopted)**: treat `Sell` as closing a long (the dominant Indian
   cash-equity case) and `Buy` as opening/increasing risk. Conservative, deterministic, and
   enforceable with the data the gate already has (`OrderSide`). This is why `check_order_fast`
   gained a required `side: OrderSide` parameter in the same change.
2. **Refuse everything under `ReduceOnly`**: strictly safer but operationally useless — the
   platform could not reduce a position through the gate at all, making reduce-only identical to
   a halt for order placement.

The side-based approximation is a **documented approximation, not a correctness guarantee**: a
`Sell` that increases a short position is not detected at the gate. Position-accurate reduction
is the responsibility of the layer that owns positions (see §4).

## 3. Collateral decision — price collar semantics (gap 0001)

The same position-free/market-data-free constraint shapes `RiskLimits::price_collar_bps`: a
market order's execution price is unknowable pre-trade, so the gate collars the **worst-case
band-edge fill** (band.upper for `Buy`, band.lower for `Sell`) against
`band.reference_price ± collar`. A fill beyond the collar is refused with
`RiskRefusal::PriceCollarViolation` (`price_collar_violation`). When no band (or a zero
reference) exists the collar is skipped — there is nothing to anchor it.

## 4. Where position-aware reduce-only lives (upstream)

Position-accurate reduce-only is enforced **before** the risk gate by the runtime REST layer:
`tantra-runtime-api-rest/src/orders.rs::state_refusal` (ADR 0018 rules 1–2) checks
`reduces_position(position, side, quantity)` — an order reduces only if it is opposite-direction
and within the held size. The contract is summarized in the workspace ADR
[`docs/design/0002-reduce-only-contract.md`](../../../docs/design/0002-reduce-only-contract.md).
A future position-aware mode inside `tantra-risk` itself would require a position port into the
pure gate — deliberately out of scope for plan 0002.

## 5. Verification

- Tests: `tests/plan0002_tests.rs` — `reduce_only_*` (buy refused / sell approved / fast path /
  decision / sizing / resume) and `price_collar_*` (within passes / beyond refused / limit
  orders untouched / no-band skip / fast-path parity).
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test --workspace --profile prod` green; `make prod` rebuilt.

## 6. Related

- `docs/design/code-engine.md` — the full pipeline (step 1.25 reduce-only, step 4.5 collar).
- `docs/design/code-refusal-state.md` — `GlobalState`/`ReduceOnlyViolation` state notes.
- `docs/design/implementation-record.md` — plan 0002 closure record.
- `docs/gaps/README.md` — gaps 0001–0004 Closed (commit `9a0bc59`).
