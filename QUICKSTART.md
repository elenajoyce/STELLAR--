# Quickstart — stellarwarden

This guide walks you from clone → build → test → example → testnet in one go.

## Prerequisites

- Rust 1.88+ with the `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-cli)

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --features opt
```

## 1. Clone

```bash
git clone https://github.com/stellar-warden-kit/stellarwarden.git
cd stellarwarden
```

## 2. Build

```bash
# Build all contracts (native, for testing)
cargo build --workspace

# Build a contract to WASM for deployment
cargo build -p denylist-gate --target wasm32-unknown-unknown --release
# Or use the workspace alias:
cargo build-wasm -p denylist-gate
```

## 3. Test

```bash
# Run the full test suite
cargo test --workspace

# Run tests for a single contract
cargo test -p denylist-gate
cargo test -p allowlist-token
cargo test -p jurisdiction-flag
```

## 4. Lint

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

## 5. Try an example

The `denylist-gate-consumer` example is the simplest composition to read:

```bash
cargo test -p denylist-gate-consumer
```

For the full RWA stack:

```bash
cargo test -p rwa-compliance-flow
```

## 6. Deploy to testnet

### Set up a testnet identity

```bash
stellar keys generate --global alice --network testnet
stellar keys fund alice --network testnet
```

### Copy the env template

```bash
cp examples/testnet.env.example .env
# Edit .env and fill in STELLAR_SOURCE, STELLAR_NETWORK, etc.
source .env
```

### Deploy a single contract

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/denylist_gate.wasm \
  --source alice \
  --network testnet
```

Or use the deploy script:

```bash
STELLAR_SOURCE=alice ./scripts/deploy-testnet.sh denylist-gate
```

### Deploy the full stack

```bash
STELLAR_SOURCE=alice ./scripts/deploy-full-stack-testnet.sh
```

See [docs/full-stack-testnet-walkthrough.md](docs/full-stack-testnet-walkthrough.md) for a complete walkthrough with invoke examples.

## 7. Web playground

Import the contract ABI into the [Stellar Lab](https://lab.stellar.org/) and invoke contract functions directly in the browser using the testnet contract IDs output by the deploy scripts.
