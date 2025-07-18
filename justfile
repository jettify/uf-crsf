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

pedantic:
  cargo clippy -- -W clippy::pedantic

audit:
  cargo audit
