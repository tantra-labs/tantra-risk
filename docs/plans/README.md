# Plans Index — `tantra-risk`

Per workspace rule `.agents/rules/gap-tracking.md`:
- Gaps are individual features (`docs/gaps/XXXX-*.md`).
- Plans are collections of gaps (`docs/plans/XXXX-*.md`).
- When implemented, plan documents are cleaned up from `docs/plans/` and recorded in this index.

## Active Plans

| # | Title | Target Scope | Gaps | Status |
|---|-------|--------------|------|--------|
| — | — | — | — | — |

## Implemented Plans

| # | Title | Target Scope | Implemented In |
|---|-------|--------------|----------------|
| 0001 | Tool-Call Annotation Hooks for the MCP Gateway (`readOnlyHint` / `destructiveHint`) | `tantra-risk` | Commit `5b00cda` (`ToolRiskHint`, `risk_classification`, `tests/tool_hints_tests.rs`) |
| 0002 | Close declared-but-unenforced risk controls | `tantra-risk` | Commit `9a0bc59` — enforce reduce-only, `price_collar_bps`, fast-path OTR parity, and configured OTR windows; tests in `tests/plan0002_tests.rs` |

## Cleanup note

The former `docs/roadmap/` was dismantled into `docs/design/` (code references,
`implementation-record.md`, `code-migration-map.md`) and `docs/gaps/` (open items). No
`docs/plans/0001-*.md` document exists for the implemented plan — `0001` is recorded directly
in the Implemented table above and was never reused for the new plan.
