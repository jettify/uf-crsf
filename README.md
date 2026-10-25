# uf-crsf

[![CI](https://github.com/jettify/uf-crsf/actions/workflows/CI.yml/badge.svg)](https://github.com/jettify/uf-crsf/actions/workflows/CI.yml)
[![codecov](https://codecov.io/gh/jettify/uf-crsf/graph/badge.svg?token=2N16CN1OZX)](https://codecov.io/gh/jettify/uf-crsf)
[![crates.io](https://img.shields.io/crates/v/uf-crsf)](https://crates.io/crates/uf-crsf)
[![docs.rs](https://img.shields.io/docsrs/uf-crsf)](https://docs.rs/uf-crsf/latest/uf_crsf/)

A `no_std` Rust library for parsing the TBS Crossfire protocol, designed for embedded environments without an allocator.

This library provides a two-layer API:

- A low-level layer for raw packet parsing from a byte stream.
- A higher-level layer that converts raw packets into idiomatic Rust structs.

## Features

- `no_std` and allocator-free for embedded systems.
- Two-layer API for flexible parsing.
- Zero copy with spill over buffer, when packet divided between two reads.
- Supports a wide range of CRSF packets.
- IO and MCU agnostic.
- Minimal dependencies.

## Note

Library is under active development and testing, API might change at any time.

## Installation

Add `uf-crsf` to your `Cargo.toml`:

```toml
[dependencies]
uf-crsf = "0.6"
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

    // A complete CRSF frame containing RC channels
    let buf: [u8; 26] = [
        0xC8, // Address
        0x18, // Length of type, payload, and CRC
        0x16, // Type (RC Channels)
        // Payload (22 bytes)
        0x03, 0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 0x60, 0x01, 0x0B, 0xF8, 0xC0,
        0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFC,
        0x42, // CRC
    ];

    let mut input = buf.as_slice();
    for item in parser.iter_packets(&mut input) {
        match item {
            Ok(p) => println!("{:?}", p),
            Err(e) => eprintln!("Error parsing packet: {:?}", e),
        }
    }
}
```


Keep the same parser across input chunks. Exhaust `iter_packets` for each chunk
so it can retain a trailing partial frame for the next chunk. Errors are yielded
as items; continue iterating after an error to resume scanning for valid frames.
If you stop early, keep the updated input cursor to resume unread bytes later.

Use `next_raw_packet` to inspect validated frames without decoding their payloads:

```rust
use uf_crsf::CrsfParser;

fn main() {
    let bytes = [
        0xC8, 0x0C, 0x14, 0x10, 0x13, 0x63, 0x97,
        0x01, 0x02, 0x03, 0x08, 0x58, 0x94, 0xFC,
    ];
    let mut parser = CrsfParser::new();

    // Chunk boundaries need not align with frame boundaries.
    for chunk in bytes.chunks(5) {
        let mut input = chunk;
        while let Some(result) = parser.next_raw_packet(&mut input) {
            match result {
                Ok(frame) => println!(
                    "type={:#04X}, payload={:?}",
                    frame.raw_packet_type(),
                    frame.payload(),
                ),
                Err(error) => eprintln!("Framing error: {:?}", error),
            }
        }
    }
}
```

Complete frames borrow directly from the input when contiguous; split frames
borrow from the parser's internal buffer. Release each raw frame before calling
`next_raw_packet` again. Keep calling until it returns `None`, even if the input
cursor is empty: recovery can yield frames or errors from retained bytes.

See the [examples](examples/README.md) for desktop serial readers, including
blocking and async wrappers and their required feature flags.


## Implementation status

**Legend:**

- `🟢` - Implemented
- `🟡` - Partial implementation or known protocol compatibility gaps
- `🔴` - Not Implemented

| Packet Name | Frame Type | Status | Notes |
| :--- | :--- | :--- | :--- |
| **Broadcast / Short Header Frames** |  |  |  |
| GPS | `0x02` | 🟢 |  |
| GPS Time | `0x03` | 🟢 |  |
| GPS Extended | `0x06` | 🟢 |  |
| Variometer Sensor | `0x07` | 🟢 |  |
| Battery Sensor | `0x08` | 🟢 |  |
| Barometric Altitude & Vertical Speed | `0x09` | 🟢 |  |
| Airspeed | `0x0A` | 🟢 |  |
| Heartbeat | `0x0B` | 🟢 |  |
| RPM | `0x0C` | 🟢 |  |
| TEMP | `0x0D` | 🟢 |  |
| Voltages | `0x0E` | 🟢 |  |
| Discontinued | `0x0F` | 🔴 |  |
| VTX Telemetry | `0x10` | 🟢 |  |
| Barometer | `0x11` | 🟢 |  |
| Magnetometer | `0x12` | 🟢 |  |
| Accel Gyro | `0x13` | 🟢 |  |
| Link Statistics | `0x14` | 🟢 |  |
| Link Statistics Repeater | `0x15` | 🟢 |  |
| RC Channels Packed Payload | `0x16` | 🟢 |  |
| Subset RC Channels Packed | `0x17` | 🔴 |  |
| RC Channels Packed 11-bits (Unused) | `0x18` | 🔴 |  |
| Link Statistics RX | `0x1C` | 🟢 | legacy 5-byte and extended 7-byte payloads |
| Link Statistics TX | `0x1D` | 🟢 | legacy 6-byte and extended 8-byte payloads |
| Attitude | `0x1E` | 🟢 |  |
| MAVLink FC | `0x1F` | 🟢 |  |
| Flight Mode | `0x21` | 🟢 |  |
| ESP_NOW Messages | `0x22` | 🟢 |  |
| Logging | `0x34` | 🟢 |  |
| ArduPilot Reserved Passthrough Frame | `0x80` | 🟢 | single, multi-packet, and status text |
| mLRS Reserved | `0x81, 0x82` | 🔴 |  |
| CRSF MAVLink Envelope | `0xAA` | 🟢 |  |
| **Extended Frames** |  |  |  |
| Parameter Ping Devices | `0x28` | 🟢 |  |
| Parameter Device Information | `0x29` | 🟢 |  |
| Parameter Settings (Entry) | `0x2B` | 🔴 |  |
| Parameter Settings (Read) | `0x2C` | 🔴 |  |
| Parameter Value (Write) | `0x2D` | 🔴 |  |
| Direct Commands | `0x32` | 🟢 | discontinued VTX `0x01`/`0x03` payloads are unspecified and unsupported |
| Remote Related Frames | `0x3A` | 🟢 | timing correction subtype `0x10` |
| Game | `0x3C` | 🟢 |  |
| MSP Request | `0x7A` | 🔴 |  |
| MSP Response | `0x7B` | 🔴 |  |
| CRSF MAVLink System Status Sensor | `0xAC` | 🟢 |  |
| **Reserved / Version-specific Frames** |  |  |  |
| Crossfire Reserved | `0x19 - 0x1B` | 🔴 |  |
| Reserved | `0x27, 0x36, 0x38, 0x3E, 0x40` | 🔴 |  |
| KISSFC Reserved | `0x78 - 0x79` | 🔴 |  |
| ArduPilot Legacy Reserved | `0x7F` | 🟢 | same payloads as `0x80` |
| Rotorflight Telemetry Envelope | `0x88` | 🔴 |  |


## License

This project is licensed under the `Apache 2.0`. See the [LICENSE](https://github.com/jettify/uf-crsf/blob/master/LICENSE) file for details.

## Protocol Specification

- [Official TBS CRSF Protocol Specification](https://github.com/tbs-fpv/tbs-crsf-spec)
- [CRSF Working Group Fork](https://github.com/crsf-wg/crsf)
