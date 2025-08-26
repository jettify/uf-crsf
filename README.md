# uf-crsf

[![Rust](https://github.com/jettify/uf-crsf/actions/workflows/CI.yml/badge.svg)](https://github.com/jettify/uf-crsf/actions/workflows/CI.yml)

A `no_std` Rust library for parsing the TBS Crossfire protocol, designed for embedded environments without an allocator.

This library provides a two-layer API:

- A low-level layer for raw packet parsing from a byte stream.
- A higher-level layer that converts raw packets into idiomatic Rust structs.

## Features

- `no_std` and allocator-free for embedded systems.
- Two-layer API for flexible parsing.
- Supports a wide range of CRSF packets.
- IO and MCU agnostic.
- Minimal dependencies.

## Note

Library is under active development and testing, API might change at any time.

## Installation

Add `uf-crsf` to your `Cargo.toml`:

```toml
[dependencies]
uf-crsf = "0.1.0"
```

Or use the command line:

```bash
cargo add uf-crsf
```

## Usage

Here is a basic example of how to parse a CRSF packet from a byte array:

```rust
use uf_crsf::CrsfParser;

fn main() {
    let mut parser = CrsfParser::new();

    // A sample CRSF packet payload for RC channels
    let buf: [u8; 26] = [
        0xC8, // Address
        0x18, // Length
        0x16, // Type (RC Channels)
        0x03, 0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 0x60, 0x01, 0x0B, 0xF8, 0xC0,
        0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 252,  // Packet
        0x42, // Crc
    ];

    for item in parser.iter_packets(&buf) {
        match item {
            Ok(p) => println!("{:?}", p),
            Err(e) => eprintln!("Error parsing packet: {:?}", e),
        }
    }
}
```

## License

This project is licensed under the `Apache 2.0`. See the [LICENSE](LICENSE) file for details.

## Protocol Specification

- [Official TBS CRSF Protocol Specification](https://github.com/tbs-fpv/tbs-crsf-spec)
- [CRSF Working Group Fork](https://github.com/crsf-wg/crsf)
