# Contributing

Thanks for contributing to `rs-auth`.

## Development

This repository is a Cargo workspace with these primary crates:

- `auth/` -> `rs-auth`
- `core/` -> `rs-auth-core`
- `pg/` -> `rs-auth-postgres`
- `axum/` -> `rs-auth-axum`

Common commands:

```bash
cargo check --workspace
cargo test --workspace
cargo fmt --all
cargo clippy --workspace --all-targets
```

## Guidelines

- Prefer small, focused changes.
- Keep public APIs explicit and well named.
- Put shared domain logic in `core/`.
- Put Postgres-specific persistence in `pg/`.
- Put Axum-specific HTTP integration in `axum/`.
- Add or update tests with behavior changes when practical.

## Release Process

This project uses [cargo-release](https://github.com/crate-ci/cargo-release) for version bumping and publishing.

### Prerequisites

```bash
cargo install cargo-release
```

### Publishing a new version

1. Ensure `CARGO_REGISTRY_TOKEN` is set (crates.io API token with publish permissions).
2. Run a dry-run first:

   ```bash
   cargo release patch --no-publish --no-push --no-tag
   ```

3. If the dry-run looks correct, execute the release:

   ```bash
   cargo release patch --execute
   ```

   This will:
   - Bump the workspace version in all `Cargo.toml` files
   - Update `Cargo.lock`
   - Create a git commit with the version bump
   - Tag the commit (`v<version>`)
   - Push the commit and tag to the remote

4. The `release.yml` GitHub Actions workflow triggers on the tag push and:
   - Runs the full verification suite (check, test, clippy, fmt)
   - Publishes crates to crates.io in dependency order: `rs-auth-core` → `rs-auth-postgres` → `rs-auth-axum` → `rs-auth`
   - Creates a GitHub Release with auto-generated notes

### Publish order

The crates **must** be published in this order because of inter-dependencies:

1. `rs-auth-core` (no workspace crate dependencies)
2. `rs-auth-postgres` (depends on core)
3. `rs-auth-axum` (depends on core)
4. `rs-auth` (depends on core, postgres, axum)

`cargo-release` and the CI workflow handle this ordering automatically.

### Required secrets

The following secret must be configured in the GitHub repository settings:

- `CARGO_REGISTRY_TOKEN` — crates.io API token with publish permissions for all four crates

## Commits

- Use clear, concise commit messages.
- Keep unrelated changes in separate commits.

## Pull Requests

- Explain the problem being solved.
- Summarize the approach taken.
- Mention follow-up work or known limitations.

## License

By contributing, you agree that your contributions will be licensed under:

- MIT license
- Apache License, Version 2.0

See `LICENSE-MIT` and `LICENSE-APACHE` for details.
