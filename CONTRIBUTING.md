# Contributing to stellarwarden

Thank you for your interest in contributing! stellarwarden is an open-source project and welcomes contributions of all kinds — bug reports, documentation improvements, new primitives, and example patterns.

---

## Getting started

1. **Fork** the repository and clone your fork locally.
2. Make sure you have Rust installed via `rustup`. The required toolchain and targets are declared in `rust-toolchain.toml`; `rustup` will pick them up automatically.
3. Verify the workspace builds and all tests pass before you start making changes:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

---

## Branch and PR flow

| Step | Detail |
|---|---|
| Branch naming | `feat/<short-description>`, `fix/<short-description>`, or `docs/<short-description>` |
| Commit messages | Use plain imperative present tense: `Add jurisdiction range check`, not `Added` or `Adding`. |
| PR title | Keep it under 70 characters. |
| PR description | Summarise what changed, how you tested it, and any follow-up work that is out of scope. |

- Open a draft PR early if you want feedback on the direction before the implementation is complete.
- Keep PRs focused. One logical change per PR is easier to review and easier to revert if something goes wrong.

---

## Code standards

- All Rust code must compile against the toolchain in `rust-toolchain.toml`.
- `cargo test --workspace` must pass with zero failures.
- `cargo clippy --workspace --all-targets -- -D warnings` must produce zero warnings.
- Format with `cargo fmt --all` before submitting.
- New contract behaviour requires a corresponding unit test in the contract's `#[cfg(test)]` module.

---

## Adding a new contract primitive

1. Create a new directory under `contracts/<name>/` with its own `Cargo.toml`.
2. Add it to the `[workspace]` members list in the root `Cargo.toml`.
3. Write a focused `src/lib.rs` — one contract, one job.
4. Add snapshot or unit tests covering the happy path and at least two error paths.
5. Document the primitive in the root `README.md` table and, if relevant, in `docs/PRIMITIVE_SELECTION_GUIDE.md`.

---

## Adding a new example

1. Create a new directory under `examples/<name>/` with its own `Cargo.toml`.
2. Add it to the `[workspace]` members list in the root `Cargo.toml`.
3. The example should demonstrate a realistic composition pattern, not just call a single primitive.
4. Include at least one integration test.
5. Add a row to the examples table in the root `README.md`.

---

## Issue labels

| Label | Meaning |
|---|---|
| `good first issue` | Self-contained, well-scoped, no deep context needed |
| `complexity: trivial` | A few lines; no design decision required |
| `complexity: medium` | Touches multiple files; needs thought about API design |
| `complexity: high` | Cross-cutting concern or new primitive; discuss in an issue first |
| `bug` | Something is broken |
| `documentation` | Docs-only change |

---

## Code of conduct

Be respectful and constructive. Discussions about design tradeoffs are encouraged; personal criticism is not. Contributors who are consistently unkind will be asked to stop.

---

## Questions?

Open a GitHub Discussion or file an issue tagged `question`.
