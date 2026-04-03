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
- Supports a wide range of CRSF packets.
- IO and MCU agnostic.
- Minimal dependencies.

## Implementation status

**Legend:**

- `🟢` - Implemented
- `🔴` - Not Implemented

| Packet Name | Packet Address | Status |
| :--- | :--- | :--- |
| **Broadcast Frames** | | |
| GPS | `0x02` | 🟢 |
| GPS Time | `0x03` | 🟢 |
| GPS Extended | `0x06` | 🟢 |
| Variometer Sensor | `0x07` | 🟢 |
| Battery Sensor | `0x08` | 🟢 |
| Barometric Altitude & Vertical Speed | `0x09` | 🟢 |
| Airspeed | `0x0A` | 🟢 |
| Heartbeat | `0x0B` | 🟢 |
| RPM | `0x0C` | 🟢 |
| TEMP | `0x0D` | 🟢 |
| Voltages | `0x0E` | 🟢 |
| Discontinued | `0x0F` | 🟢 |
| VTX Telemetry | `0x10` | 🟢 |
| Barometer | `0x11` | 🟢 |
| Magnetometer | `0x12` | 🟢 |
| Accel Gyro | `0x13` | 🟢 |
| Link Statistics | `0x14` | 🟢 |
| RC Channels Packed Payload | `0x16` | 🟢 |
| Subset RC Channels Packed | `0x17` | 🔴 |
| RC Channels Packed 11-bits | `0x18` | 🔴 |
| Link Statistics RX | `0x1C` | 🟢 |
| Link Statistics TX | `0x1D` | 🟢 |
| Attitude | `0x1E` | 🟢 |
| MAVLink FC | `0x1F` | 🟢 |
| Flight Mode | `0x21` | 🟢 |
| ESP_NOW Messages | `0x22` | 🟢 |
| **Extended Frames** | | |
| Parameter Ping Devices | `0x28` | 🟢 |
| Parameter Device Information | `0x29` | 🟢 |
| Parameter Settings (Entry) | `0x2B` | 🔴 |
| Parameter Settings (Read) | `0x2C` | 🔴 |
| Parameter Value (Write) | `0x2D` | 🔴 |
| Direct Commands | `0x32` | 🟢 |
| Logging | `0x34` | 🟢 |
| Remote Related Frames | `0x3A` | 🟢 |
| Game | `0x3C` | 🟢 |
| KISSFC Reserved | `0x78 - 0x79` | 🔴 |
| MSP Request | `0x7A` | 🔴 |
| MSP Response | `0x7B` | 🔴 |
| ArduPilot Legacy Reserved | `0x7F` | 🔴 |
| ArduPilot Reserved Passthrough Frame | `0x80` | 🟢 |
| mLRS Reserved | `0x81, 0x82` | 🔴 |
| CRSF MAVLink Envelope | `0xAA` | 🟢 |
| CRSF MAVLink System Status Sensor | `0xAC` | 🟢 |

## Note

Library is under active development and testing, API might change at any time.

## Installation

Add `uf-crsf` to your `Cargo.toml`:

```toml
[dependencies]
uf-crsf = "*" # replace * by the latest version of the crate.
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


Here is a basic example of how to parse and print telemetry `CRSF` packets from an ELRS `TX16S` radio controller, though it should work with any other `EdgeTX` device. Simply configure telemetry mirroring to a USB serial port and connect the controller to your PC.

```rust no_run
use std::env;
use std::io::ErrorKind;
use std::process::exit;
use std::time::Duration;
use uf_crsf::CrsfParser;

fn main() {
    let ports = match serialport::available_ports() {
        Ok(ports) => ports,
        Err(e) => {
            eprintln!("Failed to enumerate serial ports: {}", e);
            exit(1);
        }
    };

    if ports.is_empty() {
        eprintln!("No serial ports found.");
        eprintln!("Please specify a serial port path as an argument.");
        exit(1);
    }

    let path = env::args().nth(1).unwrap_or_else(|| {
        const DEFAULT_PORT: &str = "/dev/tty.usbmodem00000000001B1";
        if ports.iter().any(|p| p.port_name == DEFAULT_PORT) {
            println!(
                "No serial port specified. Using default port: {}",
                DEFAULT_PORT
            );
            DEFAULT_PORT.to_string()
        } else {
            println!("No serial port specified. Available ports:");
            for p in &ports {
                println!("  {}", p.port_name);
            }
            println!("\nUsing first available port: {}", ports[0].port_name);
            ports[0].port_name.clone()
        }
    });

    let mut port = serialport::new(&path, 420_000)
        .timeout(Duration::from_millis(10))
        .open()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open serial port '{}': {}", &path, e);
            exit(1);
        });

    let mut buf = [0; 1024];
    let mut parser = CrsfParser::new();
    println!("Reading from serial port '{}'...", path);
    loop {
        match port.read(buf.as_mut_slice()) {
            Ok(n) => {
                for packet in parser.iter_packets(&buf[..n]) {
                    println!("{:?}", packet);
                }
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                // This is expected when no data is coming in
            }
            Err(e) => {
                eprintln!("Error reading from serial port: {}", e);
                break;
            }
        }
    }
}

```

## License

This project is licensed under the `Apache 2.0`. See the [LICENSE](https://github.com/jettify/uf-crsf/blob/master/LICENSE) file for details.

## Protocol Specification

- [Official TBS CRSF Protocol Specification](https://github.com/tbs-fpv/tbs-crsf-spec)
- [CRSF Working Group Fork](https://github.com/crsf-wg/crsf)
