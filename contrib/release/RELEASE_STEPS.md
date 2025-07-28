# Release steps

- Bump versions in various `Cargo.toml` with `cargo release version --workspace --execute <version>`
- Update CHANGELOGs
- Commit and push: `Release vX.X.X`
- Run `just release` to build and publish all the crates
