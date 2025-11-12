# Contrib utilities

Helper scripts and docs that keep the workspace consistent.

## Scripts

Located under `contrib/scripts/` and wired into the root `justfile`.

| Script | Purpose |
| --- | --- |
| `check-fmt.sh [check]` | Format the entire workspace (default) or verify formatting when called with `check`. |
| `check-crates.sh` | Runs `cargo check`/`clippy` across the workspace with the default feature set. |
| `check-docs.sh` | Ensures `cargo doc` builds for every crate, catching broken intra-doc links. |
| `check-deny.sh` | Executes `cargo deny check` using the repo’s `deny.toml`. |
| `contributors.py` | Generates the CONTRIBUTORS list used for release notes. |

Invoke them directly (`bash contrib/scripts/check-crates.sh`) or via the `just` recipes (`just check`, `just precommit`).

## Release playbooks

`contrib/release/RELEASE_STEPS.md` contains the checklist we follow before publishing crates. Keep it updated whenever the release process changes.

## Funding + verification

- `contrib/fund` stores the assets shown on <https://rust-nostr.org/donate>.
- `contrib/verify-commits` documents the GPG/SSH verification steps used by the maintainers.
