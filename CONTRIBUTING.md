# Contributing

## Organization guidelines

This project follows the rust-nostr organization guidelines: https://github.com/rust-nostr/guidelines

## Additional repository guidelines

### Commit Style

The commit **must** be formatted as follows:

```
<context>: <short description>

<description explaining reasons for the changes>
```

If applicable, link the `issue`/`PR` to be closed with:

- Closes <url>
- Fixes <url>

The `context` **must be**:

- `nostr` for changes to the `nostr` crate
- `sdk`, `cli`, `relay-pool`, `connect`, `nwc` and so on for the others crates (remove the `nostr-` prefix)
- `test` for changes to the unit tests
- `doc` for changes to the documentation
- `contrib` for changes to the scripts and tools
- `ci` for changes to the CI code
- `refactor` for structural changes that do not change behavior

### Examples

```
nostr: add NIP32 support

Added kinds, tags and EventBuilder constructors to support NIP32.

Closes https://<domain>.com/rust-nostr/nostr/issue/1234
```

```
pool: fix connection issues

Long description...

Fixes https://<domain>.com/rust-nostr/nostr/issue/5612
```

```
nwc: add `pay_multiple_invoices` support

Closes https://<domain>.com/rust-nostr/nostr/issue/2222
```

### Coding Conventions

Install https://github.com/casey/just and use `just precommit` or `just check` 
to format and check the code before committing.
The CI also enforces this.

## Local development workflow

1. Install the workspace toolchain (`rustup show 1.85.0`) and `just`.
2. Run `just check` early—this executes formatting, clippy, doctests, and `cargo check` for every crate.
3. Use `cargo test -p <crate>` while iterating, then `cargo test --workspace` before opening a PR.
4. Keep feature-gated code tested by passing `--all-features` (or at least the features you touched).
5. When documentation changes reference code, build the docs locally with `cargo doc --workspace --no-deps`.

## Documentation contributions

- Edit Markdown in `README.md`, crate-level READMEs under `crates/*/README.md`, or the book at <https://rust-nostr.org>.
- Prefer short runnable examples; keep them in sync with the APIs by compiling them locally (use `cargo test --doc`).
- If you add scripts or tooling, document them under `contrib/README.md` so future contributors can discover them.
