# `vyapaar-risk` — Tool Risk Hints Reference

Documents `vyapaar-risk/src/tool_hints.rs`: MCP gateway integration for tool
call risk classification.

## `ToolRiskHint`

Classifies tool calls by risk level for the MCP gateway (`readOnlyHint` /
`destructiveHint` annotations):

| Hint | Meaning |
|------|---------|
| `ReadOnly` | Read-only query, no side effects |
| `Destructive` | State-changing operation, requires confirmation |
| `Emergency` | Kill-switch / emergency action, highest risk |

## `risk_classification`

Classifies a tool call into its risk hint based on tool name, parameters, and
current system state.

## Related

- Plan 0001 — Tool-Call Annotation Hooks for the MCP Gateway (`readOnlyHint` /
  `destructiveHint`), commit `5b00cda`.
- `vyapaar-agent-mcp` — MCP server consumes `ToolRiskHint` for tool annotations.
