# `vyapaar-risk` Agent Guidelines & Architecture Invariants

**Subsystem**: Gate (Pre-Trade Regulatory Risk Engine)
**Language**: Rust
**Primary Responsibility**: Sub-15 µs pre-trade regulatory validation and market microstructure enforcement.

---

## 1. Role & Architectural Hints
* `vyapaar-risk` enforces non-negotiable exchange and SEBI rules:
  1. **SEBI Algo ID Validation**: Mandatory exchange-assigned algorithmic identifier attached to every order.
  2. **Order-to-Trade Ratio (OTR) Circuit Breakers**: Sliding-window tracking of orders, modifications, cancellations, and fills with automatic halting on breach.
  3. **Scrip Circuit Bands**: 2%, 5%, 10%, 20% operational price bands rejecting out-of-band limit orders.
  4. **Single-Order Freeze Limits**: Hard quantity limits (e.g., NIFTY 1,800, BANKNIFTY 900).
  5. **Emergency Kill-Switch**: Global and per-algo immediate trading suspension and reduce-only mode.
* **Latency Invariant**: Synchronous evaluation must execute in under 15 microseconds.
* **Idea Inheritance**:
  - **NoFX**: Untrusted signal runtime clamping; the AI cannot override or bypass `vyapaar-risk`.
  - **NautilusTrader**: Synchronous deterministic risk filters before execution buffers.

---

## 2. Coding Guidelines (DDD & TDD)
* **Zero-I/O Domain Purity**: `RiskEngine` is pure and synchronous. No async/await, no network calls, no disk writes in validation paths.
* **Zero Floats Policy**: Strict decimal arithmetic via `rust_decimal::Decimal`.
* **TDD & Parallel Test Directory Rule (MANDATORY)**:
  - All unit, integration, and benchmark tests live in `tests/` parallel to `src/`, or in a `src/tests/` module directory.
  - **STRICTLY FORBIDDEN**: In-file `#[test]` blocks inside production `src/` files.
  - Extensive property tests (`proptest`) for OTR ratio edge cases, price band boundaries, and freeze bounds.

---

## 3. Development & CI Workflow
* **Linting & Formatting**:
  ```bash
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  ```
* **Testing & Benchmarks**:
  ```bash
  cargo test --all
  cargo bench
  ```
