# uf-crsf

[![Rust](https://github.com/jettify/uf-crsf/actions/workflows/CI.yml/badge.svg)](https://github.com/jettify/uf-crsf/actions/workflows/CI.yml)

This is a `no_std` Rust library for parsing the TBS Crossfire protocol. It's designed to be used in embedded environments without an allocator. The library provides a two-layer API: a low-level layer for raw packet parsing and a higher-level layer for working with idiomatic Rust structs.

## Installation

To use `uf-crsf` in your project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
uf-crsf = "0.1.0"
```

or

```bash
cargo add uf-crsf

```

## Protocol Specification

1. <https://github.com/tbs-fpv/tbs-crsf-spec>
2. <https://github.com/crsf-wg/crsf>
