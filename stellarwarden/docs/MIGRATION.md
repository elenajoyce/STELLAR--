# Migration Guide — From Hand-Rolled to stellarwarden

If you already have allowlist, denylist, or jurisdiction checks baked into your own token contract, this guide walks you through replacing them with stellarwarden primitives.

## Step 1 — Map your existing checks

Identify which compliance checks you have in your token's `transfer` function:

| Your existing check | stellarwarden primitive |
|---|---|
| `require!(allowlist.contains(from))` | `allowlist-token` |
| `require!(!denylist.contains(from))` | `denylist-gate` |
| `require!(permitted_jurisdictions.contains(jurisdiction_of(from)))` | `jurisdiction-flag` |
| Emergency pause flag | `circuit-breaker` |
| Compliance event log | `audit-log` |

## Step 2 — Deploy the primitives

Deploy each primitive you need to testnet first:

```bash
STELLAR_SOURCE=alice ./scripts/deploy-testnet.sh denylist-gate
STELLAR_SOURCE=alice ./scripts/deploy-testnet.sh jurisdiction-flag
```

## Step 3 — Initialize the primitives

```bash
# Initialize denylist-gate
stellar contract invoke \
  --id $DENYLIST_GATE_CONTRACT_ID \
  --source alice \
  --network testnet \
  -- initialize \
  --admin $(stellar keys address alice)

# Initialize jurisdiction-flag
stellar contract invoke \
  --id $JURISDICTION_FLAG_CONTRACT_ID \
  --source alice \
  --network testnet \
  -- initialize \
  --admin $(stellar keys address alice)
```

## Step 4 — Backfill data

Migrate your existing denylist/allowlist/jurisdiction data to the new contracts:

```bash
# Add existing denied addresses to denylist-gate
for ADDR in $DENIED_ADDRESSES; do
  stellar contract invoke \
    --id $DENYLIST_GATE_CONTRACT_ID \
    --source alice \
    --network testnet \
    -- deny --address "$ADDR"
done

# Set jurisdiction codes
for ENTRY in $JURISDICTION_ENTRIES; do
  ADDR=$(echo $ENTRY | cut -d: -f1)
  CODE=$(echo $ENTRY | cut -d: -f2)
  stellar contract invoke \
    --id $JURISDICTION_FLAG_CONTRACT_ID \
    --source alice \
    --network testnet \
    -- set_jurisdiction --address "$ADDR" --code "$CODE"
done
```

## Step 5 — Wire your token contract

Replace your inline checks with cross-contract calls:

```rust
// Before (inline):
if self.denylist.contains(&from) {
    panic!("denied");
}

// After (cross-contract):
denylist_gate::Client::new(&env, &denylist_gate_addr).check(&from);
```

## Step 6 — Test on testnet

Run a full transfer test against your updated contract on testnet, verifying:
- Clean addresses transfer successfully
- Denied addresses are blocked
- Jurisdiction-restricted addresses are blocked
- Circuit breaker halts all transfers when tripped

## Step 7 — Rollback plan

Keep your old inline checks behind a feature flag or `#[cfg(test)]` until you're confident in the migration. To roll back, simply remove the cross-contract calls and re-enable the inline logic.
