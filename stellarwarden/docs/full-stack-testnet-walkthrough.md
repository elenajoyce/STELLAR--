# Full-Stack Testnet Walkthrough

This guide deploys all eight stellarwarden contracts to testnet and wires them together into a working RWA compliance stack.

## Prerequisites

- Stellar CLI installed and configured
- A funded testnet identity (`alice`)
- The workspace built: `cargo build --workspace`

## 1. Deploy all contracts

```bash
STELLAR_SOURCE=alice ./scripts/deploy-full-stack-testnet.sh
source .env.testnet
```

## 2. Initialize each contract

```bash
ALICE=$(stellar keys address alice)

stellar contract invoke --id $DENYLIST_GATE_CONTRACT_ID --source alice --network testnet -- initialize --admin $ALICE
stellar contract invoke --id $JURISDICTION_FLAG_CONTRACT_ID --source alice --network testnet -- initialize --admin $ALICE
stellar contract invoke --id $CIRCUIT_BREAKER_CONTRACT_ID --source alice --network testnet -- initialize --admin $ALICE
stellar contract invoke --id $AUDIT_LOG_CONTRACT_ID --source alice --network testnet -- initialize --admin $ALICE
stellar contract invoke --id $MULTISIG_ADMIN_CONTRACT_ID --source alice --network testnet -- initialize --signers "[$ALICE]" --threshold 1
```

## 3. Set up test users

```bash
stellar keys generate bob --network testnet
stellar keys fund bob --network testnet
BOB=$(stellar keys address bob)

# Set jurisdiction for Bob
stellar contract invoke --id $JURISDICTION_FLAG_CONTRACT_ID --source alice --network testnet \
  -- set_jurisdiction --address $BOB --code '"US"'
```

## 4. Verify compliance checks

```bash
# Bob passes denylist check
stellar contract invoke --id $DENYLIST_GATE_CONTRACT_ID --source alice --network testnet \
  -- is_denied --address $BOB
# Expected: false

# Bob passes jurisdiction check
stellar contract invoke --id $JURISDICTION_FLAG_CONTRACT_ID --source alice --network testnet \
  -- is_permitted_jurisdiction --address $BOB --permitted_codes '["US","DE","GB"]'
# Expected: true
```

## 5. Trip the circuit breaker

```bash
stellar contract invoke --id $CIRCUIT_BREAKER_CONTRACT_ID --source alice --network testnet \
  -- trip

stellar contract invoke --id $CIRCUIT_BREAKER_CONTRACT_ID --source alice --network testnet \
  -- is_frozen
# Expected: true

# Reset it
stellar contract invoke --id $CIRCUIT_BREAKER_CONTRACT_ID --source alice --network testnet \
  -- reset
```

## 6. Record an audit event

```bash
stellar contract invoke --id $AUDIT_LOG_CONTRACT_ID --source alice --network testnet \
  -- record --address $BOB --event '"TRANSFER_APPROVED"'

stellar contract invoke --id $AUDIT_LOG_CONTRACT_ID --source alice --network testnet \
  -- count
# Expected: 1
```
