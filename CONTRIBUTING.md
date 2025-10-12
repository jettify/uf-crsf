# Contributing

First off, thank you for considering contributing to `uf-crsf`!

## Development Environment Setup

1. **Install Rust:** If you don't have Rust installed, you can install it via [rustup](https://rustup.rs/).

2. **Install Development Tools:** Some additional tools are used for development and maintenance.

    ```bash
    cargo install cargo-audit
    ```

    Optionally install `just` by using your favorite [package manager.](https://just.systems/man/en/packages.html#packages)

3. **Dependencies for Examples:** To run or compile examples that use a serial port, you may need to install `libudev`. On **Debian-based** systems:

    ```bash
    sudo apt install -y libudev-dev
    ```

    On **Fedora**:

    ```bash
    sudo dnf install systemd-devel
    ```

## Building the Project

To build the project, run the following command:

```bash
cargo build
```

Using `just`:

```bash
just build
```

## Running Tests

### Run all tests

To run all the tests with all features enabled:

```bash
cargo test --all-features -- --show-output
```

Using `just`:

```bash
just test
```

### Run a single test

To run a specific test, you can use the test name as a filter:

```bash
cargo test --all-features <TEST_PATTERN> -- --show-output
```

Replace `<TEST_NAME>` with the name of the test you want to run.

Using `just`:

```bash
just test <TEST_PATTERN>
```

## Running Examples

There are several examples available in the `examples` directory.

### Simple Example

This example parses a hard coded buffer.

```bash
cargo run --example simple
```

Using `just`:

```bash
just example_simple
```

### Raw Packet Example

This example parses raw packets incoming via a USB serial port.

```bash
cargo run --example raw
```

Using `just`:

```bash
just example_raw
```

### Packet Example

This example parses packets incoming via a USB serial port.

```bash
cargo run --example std
```

Using `just`:

```bash
just example_std
```
