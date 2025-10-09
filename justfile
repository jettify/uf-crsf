# List available recipes
default:
  just --list

# Run tests for all features
test:
  cargo test --all-features -- --show-output

# Run cargo build
build:
  cargo build --verbose

# Run cargo clean
clean:
  cargo clean --verbose

# Format source code with cargo fmt
fmt:
  cargo fmt --all

# Lint source code CI linter
lint:
  cargo check
  cargo clippy --all -- -D warnings

# Run example local_std that parses packets incoming via USB serial port.
example_std:
  cargo run --example=std

# Run example local_raw that parses raw packets incoming via USB serial port.
example_raw:
  cargo run --example=raw

# Lint source code with strict linter
pedantic:
  cargo clippy -- -W clippy::pedantic

# Run cargo audit to vet dependencies
audit:
  cargo audit

# Install cargo tools used in package maintenance
init_dev:
  cargo install --locked release-plz
  cargo install cargo-audit
  echo "libudev-dev required to compile/run examples that uses serialport"
  echo "Relevant package may need to be installed: sudo apt install -y libudev-dev"

# Run same testing commands as on CI server
ci:
  cargo clippy --all -- -D warnings
  cargo build --verbose
  cargo test --all-features --verbose

# Run cargo doc
doc:
  cargo doc --all-features --no-deps --open
