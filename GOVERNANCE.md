# Governance

stellarwarden is a community-maintained open-source project. This document describes how decisions are made and how maintainership works.

---

## Guiding principles

1. **Simplicity over features.** Each primitive does one job well. We resist scope creep.
2. **Correctness first.** Compliance code must be auditable. When in doubt, do less and document the limitation.
3. **Open by default.** Discussions happen in public GitHub issues and PRs, not in private channels.

---

## Roles

### Contributor

Anyone who opens a PR or files a substantive issue. No formal process required.

### Maintainer

Maintainers have merge rights and are responsible for reviewing PRs, triaging issues, and cutting releases. Maintainers are added by consensus among existing maintainers when a contributor has demonstrated consistent, high-quality contributions over time.

---

## Decision making

Most decisions (bug fixes, documentation, small new features) happen through the normal PR review process: at least one maintainer approves, no maintainer objects within a reasonable window, and the PR is merged.

For significant decisions — new primitives, breaking changes, changes to the security model — the process is:

1. Open a GitHub issue describing the proposal.
2. Allow at least one week for community feedback.
3. Reach consensus among maintainers before merging.

If maintainers cannot reach consensus, the decision is deferred until it can be resolved or the proposal is revised.

---

## Issue triage

Issues are reviewed within two weeks of filing. The triage process assigns a complexity label and either:

- Confirms it as a valid bug/feature and marks it `accepted`, or
- Requests more information, or
- Closes it as out of scope with an explanation.

Issues tagged `good first issue` are explicitly reserved as entry points for new contributors. Maintainers will not grab these unless they have been open and unclaimed for more than 60 days.

---

## Releases

Releases follow semantic versioning. Given that these are Soroban smart contracts, any change to a public contract interface is treated as a breaking change (major version bump). Patch releases are for bug fixes with no interface changes.

---

## Security

See [SECURITY.md](SECURITY.md) for the vulnerability disclosure process. Security fixes may be released out-of-band without advance notice.

---

## This document

Changes to this governance document follow the same consensus process as significant decisions described above.
