test:
  cargo test --all-features -- --show-output

build:
  cargo build
  cargo check

fmt:
  cargo fmt --all

lint:
  cargo clippy --all -- -D warnings

ex:
  cargo run --example=local_std

pedantic:
  cargo clippy -- -W clippy::pedantic

audit:
  cargo audit
