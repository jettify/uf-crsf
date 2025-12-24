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
install_dev_tools:
  cargo install --locked release-plz
  cargo install --locked cargo-audit
  cargo install --locked cargo-outdated
  cargo install --locked cargo-llvm-cov
  cargo install --locked cargo-expand
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

# Run example local_simple that parses hard coded buffer.
[group('examples')]
example_async:
  cargo run --example=async --all-features

# Lint source code with strict linter
[group('lint')]
pedantic:
  cargo clippy -- -W clippy::pedantic

# Run cargo audit to vet dependencies
[group('lint')]
audit:
  cargo audit

set positional-arguments
# Run tests for all features
[group('test')]
test args='':
  cargo test --all-features $1 -- --show-output

# Run llvm-cov code coverage tool and open report in browser
[group('test')]
cov:
  cargo llvm-cov --open

# Run same testing commands as on CI server
[group('test')]
ci:
  cargo clippy --all -- -D warnings
  cargo build --verbose
  cargo test --all-features --verbose
  cargo test --examples
