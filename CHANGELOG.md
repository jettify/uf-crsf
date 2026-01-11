# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1](https://github.com/jettify/uf-crsf/compare/v0.5.0...v0.5.1) - 2026-01-11

### Other

- Use just commands in the github actions. ([#88](https://github.com/jettify/uf-crsf/issues/88))
- Add blocking io example. ([#90](https://github.com/jettify/uf-crsf/issues/90))

## [0.5.0](https://github.com/jettify/uf-crsf/compare/v0.4.0...v0.5.0) - 2025-12-24

### Fixed

- [**breaking**] Rework embedded io implementations. ([#86](https://github.com/jettify/uf-crsf/issues/86))

### Other

- Update async example after embedded io fix/rework. ([#89](https://github.com/jettify/uf-crsf/issues/89))

## [0.4.0](https://github.com/jettify/uf-crsf/compare/v0.3.0...v0.4.0) - 2025-12-01

### Added

- Add new method for mavlink packets. ([#82](https://github.com/jettify/uf-crsf/issues/82))
- Add new method for gps and link statisticks packets. ([#81](https://github.com/jettify/uf-crsf/issues/81))
- Implement missing new method in first packets. ([#79](https://github.com/jettify/uf-crsf/issues/79))
- [**breaking**] Rework parser output type from custom to Result<Option<>>. ([#76](https://github.com/jettify/uf-crsf/issues/76))
- Implement embedded io traits integration. ([#71](https://github.com/jettify/uf-crsf/issues/71))
- Added Mavlink Sensor status packet. ([#68](https://github.com/jettify/uf-crsf/issues/68))
- Add CRSF logging packet. ([#52](https://github.com/jettify/uf-crsf/issues/52))

### Fixed

- Fixed panic when device ping packet too small. ([#59](https://github.com/jettify/uf-crsf/issues/59))
- make from_bytes safe for small buffers ([#54](https://github.com/jettify/uf-crsf/issues/54))

### Other

- Upgrade actions tools version. ([#85](https://github.com/jettify/uf-crsf/issues/85))
- Add advanced example to readme. ([#84](https://github.com/jettify/uf-crsf/issues/84))
- Add new methods for vario and vtx telemetry, add tests for vario. ([#83](https://github.com/jettify/uf-crsf/issues/83))
- Consistent interface to construct packet with new that returns Optional. ([#80](https://github.com/jettify/uf-crsf/issues/80))
- More idiomatic asserts in parser and game packet tests. ([#78](https://github.com/jettify/uf-crsf/issues/78))
- Run code coverage in same pipeline with tests. ([#75](https://github.com/jettify/uf-crsf/issues/75))
- Rename features for ebedded io. ([#74](https://github.com/jettify/uf-crsf/issues/74))
- Revert "chore!: Channge license to GPL3 ([#69](https://github.com/jettify/uf-crsf/issues/69))" ([#73](https://github.com/jettify/uf-crsf/issues/73))
- Add async example. ([#72](https://github.com/jettify/uf-crsf/issues/72))
- Add code coverage reporting. ([#70](https://github.com/jettify/uf-crsf/issues/70))
- [**breaking**] Channge license to GPL3 ([#69](https://github.com/jettify/uf-crsf/issues/69))
- Document raw packet. ([#67](https://github.com/jettify/uf-crsf/issues/67))
- Improve test coverage for gps time packet. ([#66](https://github.com/jettify/uf-crsf/issues/66))
- Add test case for buffer too small for gps extended packet. ([#65](https://github.com/jettify/uf-crsf/issues/65))
- Add basinc CONTRIBUTING file. ([#64](https://github.com/jettify/uf-crsf/issues/64))
- Add just file groupsing to make output nicer. ([#63](https://github.com/jettify/uf-crsf/issues/63))
- Improve justfile ergonomics. ([#62](https://github.com/jettify/uf-crsf/issues/62))
- Add test case for esp and flight mode packet for buffer too small ([#61](https://github.com/jettify/uf-crsf/issues/61))
- Add test cases when buffer to small for Game packet. ([#60](https://github.com/jettify/uf-crsf/issues/60))
- Cleanup comments, add dedicated section for examples. ([#58](https://github.com/jettify/uf-crsf/issues/58))
- Improve test coverage for battery packet. ([#57](https://github.com/jettify/uf-crsf/issues/57))
- Improve error handling for baro altitude packet. ([#56](https://github.com/jettify/uf-crsf/issues/56))
- Add test cases for errors in attitude packets. ([#55](https://github.com/jettify/uf-crsf/issues/55))

## [0.3.0](https://github.com/jettify/uf-crsf/compare/v0.2.1...v0.3.0) - 2025-09-06

### Fixed

- [**breaking**] Remove heapless from public field to ensure better compatibility with downstream users. ([#47](https://github.com/jettify/uf-crsf/issues/47))

### Other

- Indicate implemented packets in readme. ([#51](https://github.com/jettify/uf-crsf/issues/51))
- Bump version of heapless. ([#50](https://github.com/jettify/uf-crsf/issues/50))
- Run security audit on schedule instead on each PR. ([#46](https://github.com/jettify/uf-crsf/issues/46))
- [**breaking**] Remove heapless::String from public API. ([#48](https://github.com/jettify/uf-crsf/issues/48))

## [0.2.1](https://github.com/jettify/uf-crsf/compare/v0.2.0...v0.2.1) - 2025-09-04

### Fixed

- Make library heapless 0.8 and 0.9 compatible. ([#45](https://github.com/jettify/uf-crsf/issues/45))
- Adds comprehensive bounds checks to all `write_to` methods. ([#43](https://github.com/jettify/uf-crsf/issues/43))

### Other

- Fixed homepage links in Cargo.toml, update README with badges. ([#44](https://github.com/jettify/uf-crsf/issues/44))
- Add full text of Apache license. ([#41](https://github.com/jettify/uf-crsf/issues/41))

## [0.2.0](https://github.com/jettify/uf-crsf/compare/v0.1.0...v0.2.0) - 2025-08-28

### Added

- Add experimental crossfire commands ([#39](https://github.com/jettify/uf-crsf/issues/39))
## [0.1.0] - 2025-08-27

### Features

- Initial release
