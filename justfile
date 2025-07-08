test:
  cargo test -- --show-output

build:
  cargo build
  cargo check

fmt:
  cargo fmt --all

lint:
  cargo clippy --all -- -D warnings

ex:
  cargo run --example=local_std
