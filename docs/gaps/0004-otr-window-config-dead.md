# 0004 — OTR window config knobs (`window_1min_ms` / `window_5min_ms`) never consulted

**Status**: open

## What's stubbed or hardcoded

`OtrConfig` (`src/otr.rs`) declares two window fields:

- `window_1min_ms: u64` (default `60_000`)
- `window_5min_ms: u64` (default `300_000`)

Neither field is ever read. `OtrTracker::window_otr_1min` calls
`self.window_otr(now, 60_000)` and `window_otr_5min` calls `self.window_otr(now, 300_000)` —
the window durations are **hardcoded** in the methods. The engine (`src/engine.rs`) only ever
calls `window_otr_1min`; `window_otr_5min` is not used by any enforcement path. Therefore
configuring `OtrConfig { window_1min_ms: X, window_5min_ms: Y }` has no observable effect.

Source-verified: `window_1min_ms` / `window_5min_ms` appear only in `src/otr.rs` (field
declarations + `Default`); `src/engine.rs` references neither field and never calls
`window_otr_5min`.

## Why

`OtrConfig` is a serde/JSON-schema config surface — operators can (and will) set these knobs
believing they tune the enforcement windows. Today the 1-minute enforcement window is fixed,
the 5-minute medium-term window is computed-but-unused, and the two config fields are dead
configuration. This is a correctness trap: a user can widen `window_1min_ms` expecting a
looser OTR and silently get the hardcoded 60-second window.

## How to close it

1. Make `window_otr` methods read the config: either pass the window from `OtrConfig` into
   the tracker calls, or have the tracker carry the configured windows and use them in
   `window_otr_1min` / `window_otr_5min` (keep the 1-min/5-min convenience methods' names, or
   make them take `window_ms` directly from `OtrConfig`).
2. Wire the 5-minute window into enforcement if a medium-term OTR tier is desired; otherwise
   remove `window_5min_ms` / `window_otr_5min` or document them as analysis-only helpers so
   they are not mistaken for enforced controls.
3. Add tests in `tests/` (Red → Green): with a customized `window_1min_ms`, a placement that
   is inside the *configured* window but outside 60s is enforced per the config; `window_otr_5min`
   honors the configured 5-min window.

Related: collected under `docs/plans/0002-close-declared-unenforced-risk-controls.md`.
