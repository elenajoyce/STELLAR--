# stellarwarden

> On-chain compliance contracts for Stellar — allowlist/denylist token gating and jurisdiction checks for RWA and stablecoin issuers, built on Soroban.

[![CI](https://github.com/elenajoyce/stellarwarden/actions/workflows/ci.yml/badge.svg)](https://github.com/elenajoyce/stellarwarden/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## The problem

RWA and stablecoin issuers building on Stellar/Soroban all end up needing the same handful of compliance gates — allowlists, denylists, jurisdiction restrictions — before they can legally let a token move. Today every issuer hand-rolls this logic inside their own token contract, which means more audit surface, more chances for a subtle bug to become a compliance incident, and no shared, reviewed reference to build on.

**stellarwarden** gives the ecosystem a small set of standardized, auditable, independently testable contracts for exactly these checks.

---

## What's in this repo

This is a Cargo workspace with nine focused Soroban contracts, each doing one job:

| Contract | Description |
|---|---|
| `allowlist-token` | Token wrapper that only permits transfers between two addresses that are both present on an on-chain allowlist. Admin-managed; forwards cleared transfers to an underlying SEP-41 token contract. |
| `denylist-gate` | Standalone denylist other contracts call via cross-contract invocation (`check(address)`) before executing a transfer. |
| `jurisdiction-flag` | Attaches an issuer-controlled jurisdiction code (e.g. ISO country code) to an address, with `is_permitted_jurisdiction` for transfer gating. |
| `audit-log` | On-chain, append-only audit trail for compliance events, queryable by other contracts. |
| `circuit-breaker` | Admin-controlled emergency freeze switch other contracts check before allowing a transfer. |
| `compliance-aggregator` | Batches denylist and jurisdiction checks into a single cross-contract call with per-check breakdown. |
| `multisig-admin` | M-of-N multisig authorization, usable as the admin address of any primitive. |
| `pausable` | Shared pause/unpause helper library composed into other contracts at compile time. Not a deployable contract. |
| `policy-engine` | Composes denylist and jurisdiction checks with configurable AND/OR logic into a single `evaluate` call. |

### Examples

Each example under `/examples` demonstrates a different composition pattern:

| Example | Pattern |
|---|---|
| `circuit-breaker-policy-engine` | Wires circuit-breaker as a pre-check before policy evaluation |
| `compliance-integration-flow` | Combines jurisdiction, multisig circuit breaker, and audit log |
| `denylist-gate-consumer` | Minimal token calling denylist-gate + circuit-breaker |
| `denylist-gate-sep41` | Denylist-gate integration for SEP-41 anchor compliance |
| `jurisdiction-flag-consumer` | Token enforcing jurisdiction-based restrictions |
| `jurisdiction-denylist-consumer` | Combines jurisdiction and denylist checks |
| `multisig-governed-allowlist` | Multisig as admin of allowlist-token |
| `rwa-compliance-flow` | Full RWA stack: circuit breaker + allowlist + denylist + jurisdiction + audit |
| `rwa-token` | Reference RWA token composing all three core contracts |

---

## Quick start

New here? Start with [QUICKSTART.md](QUICKSTART.md).

```bash
# Clone
git clone https://github.com/elenajoyce/stellarwarden.git
cd stellarwarden

# Run the full test suite
cargo test --workspace

# Lint
cargo clippy --workspace --all-targets -- -D warnings

# Build a single contract to WASM
cargo build -p denylist-gate --target wasm32-unknown-unknown --release

# Or use the alias
cargo build-wasm -p denylist-gate
```

See `examples/testnet.env.example` for identity/network setup needed for testnet deployments.

### Deploy a contract to testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/denylist_gate.wasm \
  --source <your-testnet-identity> \
  --network testnet
```

Use `scripts/deploy-testnet.sh` to build and deploy one contract in a single step:

```bash
STELLAR_SOURCE=<your-testnet-identity> ./scripts/deploy-testnet.sh denylist-gate
```

To deploy all eight deployable contracts:

```bash
STELLAR_SOURCE=<your-testnet-identity> ./scripts/deploy-full-stack-testnet.sh
```

---

## Working with a single contract

```bash
# Test just one contract
cargo test -p allowlist-token

# Lint just one contract
cargo clippy -p denylist-gate --all-targets -- -D warnings

# Build just one contract to WASM
cargo build -p jurisdiction-flag --target wasm32-unknown-unknown --release
```

---

## Architecture

These contracts are building blocks, not standalone products. They're meant to be composed into a real token or RWA wrapper contract via cross-contract calls — for example, a token's `transfer` function calling `denylist-gate.check(address)` before moving funds, or checking `jurisdiction-flag.is_permitted_jurisdiction(address, allowed_codes)` before permitting a transaction.

See `/examples/denylist-gate-consumer` for a worked example. `allowlist-token` is the one exception — it's a thin wrapper you can deploy in front of an existing SEP-41 token.

> **Threat model note:** The wrapper cannot stop a holder from calling the underlying token directly. The underlying token must enforce wrapper-only transfers via `AUTH_REQUIRED`/`AUTH_REVOCABLE` flags and a custody design.

---

## Choosing which contracts to compose

See [docs/PRIMITIVE_SELECTION_GUIDE.md](docs/PRIMITIVE_SELECTION_GUIDE.md) for profiles of common RWA and stablecoin use cases and which contract combinations fit each.

## Migrating from hand-rolled compliance

See [docs/MIGRATION.md](docs/MIGRATION.md) for a step-by-step guide to replacing hand-rolled allowlist, denylist, or jurisdiction checks with these contracts.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the fork → branch → PR flow, and [GOVERNANCE.md](GOVERNANCE.md) for how issues are triaged.

Issues are labeled by complexity (`complexity: trivial`, `complexity: medium`, `complexity: high`). Issues tagged `good first issue` are a good place to start.

This repo is part of the **Drips Wave Stellar Program**.

## Security

See [SECURITY.md](SECURITY.md) for how to report a vulnerability.

## License

[MIT](LICENSE)
