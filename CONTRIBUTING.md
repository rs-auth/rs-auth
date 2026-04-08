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
