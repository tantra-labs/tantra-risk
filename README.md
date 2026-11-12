# व्यापार・风控 tantra-risk

> *Regulatory Risk Gate: Sub-15 µs pre-trade risk engine for Indian markets (NSE/BSE).*

Pre-trade compliance and circuit breakers complying with SEBI algorithmic trading guidelines.

## Capabilities

- Sub-15 µs pre-trade checks
- SEBI Algo ID verification and audit enforcement
- Order-to-Trade Ratio (OTR) circuit breakers
- Dynamic scrip price band clamping and freeze quantity limits
- Instant master kill-switch

## Build & Test

```bash
make prod
make check
make test
```

License: AGPL-3.0.
