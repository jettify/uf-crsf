# List available recipes
default:
  just --list

# Run cargo doc
[group('build')]
doc:
  cargo doc --all-features --no-deps --open

# Run cargo build
[group('build')]
build:
  cargo build --verbose

# Run cargo clean
[group('build')]
clean:
  cargo clean --verbose

# Install cargo tools used in package maintenance
[group('build')]
init_dev:
  cargo install --locked release-plz
  cargo install cargo-audit
  echo "libudev-dev required to compile/run examples that uses serialport"
  echo "Relevant package may need to be installed: sudo apt install -y libudev-dev"

# Format source code with cargo fmt
[group('lint')]
fmt:
  cargo fmt --all

# Lint source code CI linter
[group('lint')]
lint:
  cargo check
  cargo clippy --all -- -D warnings

# Run example local_std that parses packets incoming via USB serial port.
[group('examples')]
example_std:
  cargo run --example=std

# Run example local_raw that parses raw packets incoming via USB serial port.
[group('examples')]
example_raw:
  cargo run --example=raw

# Run example local_simple that parses hard coded buffer.
[group('examples')]
example_simple:
  cargo run --example=simple

# Lint source code with strict linter
[group('lint')]
pedantic:
  cargo clippy -- -W clippy::pedantic

# Run cargo audit to vet dependencies
[group('lint')]
audit:
  cargo audit

# Run tests for all features
[group('test')]
test:
  cargo test --all-features -- --show-output

# Run same testing commands as on CI server
[group('test')]
ci:
  cargo clippy --all -- -D warnings
  cargo build --verbose
  cargo test --all-features --verbose
