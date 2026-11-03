# Security Policy

## Supported versions

| Version | Supported |
|---|---|
| `main` branch | ✅ Active |
| Older tagged releases | ⚠️ Best-effort only |

---

## Reporting a vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Report vulnerabilities by emailing the maintainers directly. Include:

- A clear description of the vulnerability and its potential impact.
- Steps to reproduce or a proof-of-concept (even a rough one helps).
- The affected contracts and versions.
- Whether you believe the issue is exploitable on testnet or mainnet.

We will acknowledge receipt within **48 hours** and aim to provide a status update within **5 business days**.

---

## Disclosure policy

stellarwarden follows responsible disclosure:

1. You report the vulnerability privately.
2. We reproduce, triage, and develop a fix.
3. We coordinate a release date with you.
4. We publish the fix and a security advisory at the same time.
5. You may publish your findings after the fix is released.

We ask for a **90-day embargo** from first report to public disclosure, which can be shortened by mutual agreement if the fix is straightforward, or extended if the issue is particularly complex.

---

## Scope

The following are in scope for this policy:

- All contracts under `contracts/`
- All examples under `examples/` (particularly any that handle token transfers or admin auth)
- Logic errors that could allow unauthorized transfers, admin bypass, or denylist/allowlist circumvention

The following are out of scope:

- Theoretical issues with no practical exploit path on Soroban
- Issues in the Soroban SDK itself (report those to [Stellar's security team](https://stellar.org/security))
- Issues requiring the attacker to already hold admin keys

---

## Threat model note

These contracts are building blocks. The threat model assumes:

- The admin private key is kept secure by the deployer.
- `allowlist-token` cannot prevent a holder from calling the underlying token contract directly unless the underlying token enforces `AUTH_REQUIRED`/`AUTH_REVOCABLE` and a custody design at the asset level.
- Cross-contract invocations trust the return values of the called contract. Composing these primitives with untrusted third-party contracts is outside the scope of this project's security guarantees.

---

## Bug bounty

There is currently no formal bug bounty program. We are grateful for responsible disclosures and will publicly acknowledge security researchers who report valid issues (unless they prefer anonymity).
