#!/usr/bin/env bash
# Deploy the full stellarwarden stack to Stellar testnet.
#
# Usage:
#   STELLAR_SOURCE=<identity> ./scripts/deploy-full-stack-testnet.sh
#
# Outputs a .env file with all deployed contract IDs.

set -euo pipefail

STELLAR_NETWORK="${STELLAR_NETWORK:-testnet}"
OUTPUT_ENV="${OUTPUT_ENV:-.env.testnet}"

if [[ -z "${STELLAR_SOURCE:-}" ]]; then
  echo "Error: STELLAR_SOURCE is not set."
  exit 1
fi

CONTRACTS=(
  allowlist-token
  denylist-gate
  jurisdiction-flag
  audit-log
  circuit-breaker
  compliance-aggregator
  multisig-admin
  policy-engine
)

echo "==> Building all contracts..."
for CONTRACT in "${CONTRACTS[@]}"; do
  cargo build -p "${CONTRACT}" --target wasm32-unknown-unknown --release
done

echo "" > "${OUTPUT_ENV}"
echo "# stellarwarden testnet deployment — $(date)" >> "${OUTPUT_ENV}"
echo "STELLAR_NETWORK=${STELLAR_NETWORK}" >> "${OUTPUT_ENV}"
echo "STELLAR_SOURCE=${STELLAR_SOURCE}" >> "${OUTPUT_ENV}"
echo "" >> "${OUTPUT_ENV}"

echo "==> Deploying contracts to ${STELLAR_NETWORK}..."
for CONTRACT in "${CONTRACTS[@]}"; do
  WASM_NAME="${CONTRACT//-/_}.wasm"
  WASM_PATH="target/wasm32-unknown-unknown/release/${WASM_NAME}"
  echo -n "  Deploying ${CONTRACT}... "
  CONTRACT_ID=$(stellar contract deploy \
    --wasm "${WASM_PATH}" \
    --source "${STELLAR_SOURCE}" \
    --network "${STELLAR_NETWORK}")
  ENV_KEY="$(echo "${CONTRACT}" | tr '[:lower:]-' '[:upper:]_')_CONTRACT_ID"
  echo "${ENV_KEY}=${CONTRACT_ID}" >> "${OUTPUT_ENV}"
  echo "✅ ${CONTRACT_ID}"
done

echo ""
echo "✅ Full stack deployed. Contract IDs written to ${OUTPUT_ENV}"
echo "   Source it with: source ${OUTPUT_ENV}"
