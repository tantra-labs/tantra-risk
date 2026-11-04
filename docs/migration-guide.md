# `vyapaar-risk` — Subagent Migration & DDD Refactoring Guide

> **Tier**: Tier 1A (Depends only on `vyapaar-core`)
> **Source**: `/home/pmallapp/Devel/Finance/honba-workspace/honba-mon`
> **Target**: `/home/pmallapp/Devel/Finance/vyapaar-labs/vyapaar-risk`

---

## 1. File-by-File Migration Map

| Source Path (`honba-mon/`) | Target Path (`vyapaar-risk/`) | Action & DDD Refactoring Required |
| :--- | :--- | :--- |
| `Cargo.toml` | `Cargo.toml` | Ensure `vyapaar-core = { path = "../vyapaar-core" }` and all dependencies (`rust_decimal`, `serde`, `chrono`, `thiserror`, `tracing`, `schemars`) match. |
| `Makefile` | `Makefile` | Port `Makefile` and replace `honba-mon` with `vyapaar-risk`. |
| `src/lib.rs` | `src/lib.rs` | Port module declarations and public re-exports; update doc comments from `honba-mon` / `honba_core` to `vyapaar-risk` / `vyapaar_core`. |
| `src/algo_registry.rs` | `src/algo_registry.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/engine.rs` | `src/engine.rs` | Replace `honba_core` with `vyapaar_core`. Verify zero heap allocations on `check_order_fast`. |
| `src/error.rs` | `src/error.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/limits.rs` | `src/limits.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/metrics.rs` | `src/metrics.rs` | Port atomic Prometheus histogram and counters. |
| `src/otr.rs` | `src/otr.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/refusal.rs` | `src/refusal.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/sizing_gate.rs` | `src/sizing_gate.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/state.rs` | `src/state.rs` | Replace `honba_core` with `vyapaar_core`. |
| `tests/*.rs` | `tests/*.rs` | Copy all test files from `honba-mon/tests/` into `vyapaar-risk/tests/`, replacing `honba_mon` → `vyapaar_risk` and `honba_core` → `vyapaar_core`. Extract any inline `#[cfg(test)]` from `src/*.rs` into `tests/`. |

---

## 2. Strict Invariants Checklist
- [x] **Zero-I/O**: No `tokio`, no network/file I/O, no `Utc::now()` in `src/`.
- [x] **Zero Inline Tests**: `grep -rn "#\[cfg(test)\]" src/` must return 0 matches.

## 3. Verification Commands
```bash
cd /home/pmallapp/Devel/Finance/vyapaar-labs/vyapaar-risk
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
