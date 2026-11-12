# Gaps Index — `tantra-risk`

Per workspace rule `.agents/rules/gap-tracking.md`:

- Gaps are individual missing features / declared-but-unenforced controls / stubs
  (`docs/gaps/XXXX-*.md`).
- Plans are collections of gaps (`docs/plans/XXXX-*.md`).
- When implemented and verified, a gap document is removed from this index and recorded
  as Closed (with the closing commit/PR) in the table below.

This directory was created when the former `docs/roadmap/` was dismantled: completed
milestones moved to `docs/design/implementation-record.md` and the file mapping to
`docs/design/code-migration-map.md`; remaining declared-but-unenforced items were filed here
as gaps. Every gap below was verified against `src/` on the filing date (2026-11-12).

## Open Gaps

| # | Title | Severity | Status | Plan |
|---|-------|----------|--------|------|
| 0001 | Market-order price collar (`price_collar_bps`) declared but never enforced | Medium | open | [0002](../plans/0002-close-declared-unenforced-risk-controls.md) |
| 0002 | Reduce-only mode (`GlobalState::ReduceOnly`) not enforced by any check path | High | open | [0002](../plans/0002-close-declared-unenforced-risk-controls.md) |
| 0003 | `check_order_fast` skips per-scrip OTR check, warning zone, and cooldown auto-resume | Medium | open | [0002](../plans/0002-close-declared-unenforced-risk-controls.md) |
| 0004 | OTR window config knobs (`window_1min_ms` / `window_5min_ms`) never consulted | Low | open | [0002](../plans/0002-close-declared-unenforced-risk-controls.md) |

## Closed Gaps

| # | Title | Severity | Status | Closed In |
|---|-------|----------|--------|-----------|
| — | — | — | — | — |

## Verification

Each gap was filed only after grepping `src/` for the exact identifiers. There are no
`TODO` / `FIXME` / `stub` / `unimplemented` markers anywhere in `src/` (verified
2026-11-12); the gaps below are behavioral — declared configuration or API surface that no
check path actually enforces.
