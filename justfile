default: lint build test

test:
  cargo test --all-features -- --show-output

build:
  cargo build
  cargo check

fmt:
  cargo fmt --all

lint:
  cargo clippy --all -- -D warnings

ex_std:
  cargo run --example=local_std

ex_raw:
  cargo run --example=local_raw

# Run more strict linter
pedantic:
  cargo clippy -- -W clippy::pedantic

audit:
  cargo audit

# Install cargo tools used in package maitanance
init_dev:
  cargo install git-cliff
  cargo install cargo-bloat
  cargo install cargo-audit
