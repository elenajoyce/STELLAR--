# Primitive Selection Guide

This guide helps you choose which stellarwarden primitives to compose for common RWA and stablecoin use cases.

## Use case profiles

### 1. Regulated stablecoin (e.g. USDC-like)

**Requirements:** Block sanctioned addresses, emergency freeze capability.

**Recommended primitives:**
- `denylist-gate` — block OFAC-sanctioned / fraudulent addresses
- `circuit-breaker` — emergency pause in case of exploit or regulatory order
- `audit-log` — on-chain record for compliance reporting
- `multisig-admin` — require M-of-N compliance officer signatures for admin actions

### 2. Tokenized real estate (RWA)

**Requirements:** Restrict to KYC'd investors, jurisdiction gating, allowlist-only transfers.

**Recommended primitives:**
- `allowlist-token` — only KYC'd investors can hold/transfer
- `jurisdiction-flag` — restrict to permitted jurisdictions (e.g. accredited investor countries)
- `denylist-gate` — block any subsequently disqualified investors
- `audit-log` — immutable transfer history for regulatory audit
- `multisig-admin` — M-of-N for allowlist changes (prevents single point of failure)

### 3. Tokenized fund / private credit

**Requirements:** Sophisticated investor restrictions, jurisdiction compliance, emergency controls.

**Recommended primitives:**
- `compliance-aggregator` — single call batches denylist + jurisdiction checks
- `circuit-breaker` — halt redemptions/transfers during NAV recalculation
- `policy-engine` (AND mode) — require both denylist and jurisdiction checks to pass
- `audit-log` — record all transfer approvals

### 4. Cross-border payments / stablecoin rails

**Requirements:** Jurisdiction screening, emergency freeze, SEP-41 compatibility.

**Recommended primitives:**
- `denylist-gate-sep41` example — drop-in SEP-41 wrapper with denylist
- `jurisdiction-flag` — screen sender/receiver jurisdictions
- `circuit-breaker` — emergency halt for FX or liquidity events

### 5. Minimal / low-overhead token

**Requirements:** Basic sanctions screening only.

**Recommended primitives:**
- `denylist-gate` alone — single cross-contract call, minimal overhead

## Composition matrix

| Primitive | Stablecoin | RWA | Fund | Payments | Minimal |
|---|---|---|---|---|---|
| `allowlist-token` | — | ✅ | ✅ | — | — |
| `denylist-gate` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `jurisdiction-flag` | — | ✅ | ✅ | ✅ | — |
| `audit-log` | ✅ | ✅ | ✅ | — | — |
| `circuit-breaker` | ✅ | — | ✅ | ✅ | — |
| `compliance-aggregator` | — | — | ✅ | ✅ | — |
| `multisig-admin` | ✅ | ✅ | ✅ | — | — |
| `policy-engine` | — | — | ✅ | — | — |
