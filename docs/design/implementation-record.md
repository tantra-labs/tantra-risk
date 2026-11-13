# `tantra-risk` — Implementation Record

Dismantled from the former `docs/roadmap/` (`roadmap.md` + `migration-verification.md`).
Completed milestones are recorded here; remaining future work moved to
[`docs/gaps/`](../gaps/README.md) and [`docs/plans/`](../plans/README.md).

## Migration: `honba-mon` → `tantra-risk` (Tier 1A)

| Milestone | Status |
|-----------|--------|
| Module 1: Core Risk Engine & SEBI Algo Registry | Complete |
| Module 2: Dynamic Sizing Gate & Observability Metrics | Complete |
| Module 3: Property-Based & Latency Verification | Complete |
| Zero inline `#[cfg(test)]` in `src/` (all tests in `tests/`) | Complete |
| `cargo fmt --check` | Clean |
| `cargo clippy --all-targets -- -D warnings` | Clean |
| `cargo test` | Passing |

The file-by-file mapping is in [`code-migration-map.md`](code-migration-map.md).

## Subsequent landings

| Landing | Status |
|---------|--------|
| Plan 0001 — Tool-Call Annotation Hooks for the MCP Gateway (`ToolRiskHint`, `risk_classification`, `tests/tool_hints_tests.rs`) | Complete, commit `5b00cda` |
| Docs: code reference docs, migration map, implementation record | Complete (this change) |
| Docs: former `docs/roadmap/` dismantled into `design/` / `gaps/` / `plans/` | Complete (this change) |

## Verification (docs batch)

```bash
# Run from the repository root
grep -rn "#\[cfg(test)\]" src/        # zero inline tests in production source
grep -rn "TODO\|FIXME\|stub\|unimplemented" src/   # zero matches
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test                            # --profile prod in this workspace's canonical profile
```

Test inventory (all under `tests/`, parallel to `src/`):

- `risk_tests.rs` — kill switch, per-algo halt, price bands, market circuit, SEBI algo ID,
  OTR circuit breaker, order rate limiter, scrip OTR, freeze slicing, cooldown auto-resume.
- `otr_tests.rs` — cumulative & sliding-window OTR, grace period, warning-zone boundaries,
  5-min window eviction, cancellations/modifications, fills.
- `registry_tests.rs` — register / suspend / reactivate / unregister.
- `phase7_tests.rs` — sizing gate, Prometheus metrics + OpenTelemetry `TraceContext`, freeze
  slices / concentration / MIS leverage / fast-check refusal variants.
- `proptest_tests.rs` — property-based randomized order streams and edge cases
  (regressions captured in `tests/proptest_tests.proptest-regressions`).
- `tool_hints_tests.rs` — exhaustive `risk_classification` and `mcp_key()` tests.
- `plan0002_tests.rs` — plan 0002 (2026-11-13): reduce-only enforcement on every path,
  market-order price collar, `check_order_fast` per-scrip OTR / cooldown parity, and
  configured OTR windows (15 tests).

## Closed gaps

All previously open gaps are closed (2026-11-13, plan 0002, commit `9a0bc59`):
unenforced `price_collar_bps` (0001), unenforced reduce-only mode (0002),
`check_order_fast` scrip-OTR divergence (0003), and dead OTR window config knobs (0004).
The `docs/gaps/README.md` Closed table records each with its closing commit. The reduce-only
and price-collar semantics are recorded as a design decision in
[`code-reduce-only.md`](code-reduce-only.md) (workspace ADR
`docs/design/0002-reduce-only-contract.md`).
