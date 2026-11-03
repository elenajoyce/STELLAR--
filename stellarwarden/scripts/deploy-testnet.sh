#!/usr/bin/env bash
# Deploy a single stellarwarden contract to Stellar testnet.
#
# Usage:
#   STELLAR_SOURCE=<identity> ./scripts/deploy-testnet.sh <contract-name>
#
# Example:
#   STELLAR_SOURCE=alice ./scripts/deploy-testnet.sh denylist-gate
#
# Environment variables:
#   STELLAR_SOURCE   — Stellar CLI identity name (required)
#   STELLAR_NETWORK  — Network to deploy to (default: testnet)

set -euo pipefail

CONTRACT="${1:-}"
STELLAR_NETWORK="${STELLAR_NETWORK:-testnet}"

if [[ -z "$CONTRACT" ]]; then
  echo "Usage: $0 <contract-name>"
  echo "Available contracts:"
  echo "  allowlist-token  denylist-gate  jurisdiction-flag  audit-log"
  echo "  circuit-breaker  compliance-aggregator  multisig-admin  policy-engine"
  exit 1
fi

if [[ -z "${STELLAR_SOURCE:-}" ]]; then
  echo "Error: STELLAR_SOURCE environment variable is not set."
  echo "Set it to your Stellar CLI identity name, e.g.:"
  echo "  STELLAR_SOURCE=alice $0 $CONTRACT"
  exit 1
fi

echo "==> Building ${CONTRACT} to WASM..."
cargo build -p "${CONTRACT}" --target wasm32-unknown-unknown --release

WASM_NAME="${CONTRACT//-/_}.wasm"
WASM_PATH="target/wasm32-unknown-unknown/release/${WASM_NAME}"

if [[ ! -f "$WASM_PATH" ]]; then
  echo "Error: WASM file not found at ${WASM_PATH}"
  exit 1
fi

echo "==> Deploying ${CONTRACT} to ${STELLAR_NETWORK}..."
CONTRACT_ID=$(stellar contract deploy \
  --wasm "${WASM_PATH}" \
  --source "${STELLAR_SOURCE}" \
  --network "${STELLAR_NETWORK}")

echo ""
echo "✅ Deployed ${CONTRACT} to ${STELLAR_NETWORK}"
echo "   Contract ID: ${CONTRACT_ID}"
echo ""
echo "Save this for your .env:"
echo "  $(echo "${CONTRACT}" | tr '[:lower:]-' '[:upper:]_')_CONTRACT_ID=${CONTRACT_ID}"
