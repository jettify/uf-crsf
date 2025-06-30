# uf-crsf

Protocol parser for `no_std` environment for TBS Crossfire used in
RC links like ExpresssLRS.

**Note:** This library is a work in progress.

## CRSF Protocol Message Implementation Status

| Frame Type Hex | Message Name                       | Implemenation Status |
| :------------- | :--------------------------------- | :------------------- |
| `0x02`         | `GPS`                              | [x] |
| `0x03`         | `GPS Time`                         | [x] |
| `0x06`         | `GPS Extended`                     | [x] |
| `0x07`         | `Variometer Sensor`                | [x] |
| `0x08`         | `Battery Sensor`                   | [x] |
| `0x09`         | `Barometric Altitude & Vertical Speed` | [x]  |
| `0x0A`         | `Airspeed`                         | [x] |
| `0x0B`         | `Heartbeat`                        | [x] |
| `0x0C`         | `RPM`                              | [] |
| `0x0D`         | `TEMP`                             | [] |
| `0x0E`         | `Voltages (or "Voltage Group")`    | [] |
| `0x0F`         | `Discontinued`                     | [] |
| `0x10`         | `VTX Telemetry`                    | [] |
| `0x14`         | `Link Statistics`                  | [x] |
| `0x16`         | `RC Channels Packed Payload`       | [] |
| `0x17`         | `Subset RC Channels Packed`        | [x] |
| `0x18`         | `RC Channels Packed 11-bits (Unused)` | []  |
| `0x19`-`0x1B`  | `Reserved Crossfire`               | [] |
| `0x1C`         | `Link Statistics RX`               | [] |
| `0x1D`         | `Link Statistics TX`               | [] |
| `0x1E`         | `Attitude`                         | [] |
| `0x1F`         | `MAVLink FC`                       | [] |
| `0x21`         | `Flight Mode`                      | [] |
| `0x22`         | `ESP_NOW Messages`                 | [] |
| `0x27`         | `Reserved`                         | [] |
| `0x28`         | `Parameter Ping Devices`           | [] |
| `0x29`         | `Parameter Device Information`     | [] |
| `0x2B`         | `Parameter Settings (Entry)`       | [] |
| `0x2C`         | `Parameter Settings (Read)`        | [] |
| `0x2D`         | `Parameter Value (Write)`          | [] |
| `0x32`         | `Direct Commands`                  | [] |
| `0x34`         | `Logging`                          | [] |
| `0x36`         | `Reserved`                         | [] |
| `0x38`         | `Reserved`                         | [] |
| `0x3A`         | `Remote Related Frames`            | [] |
| `0x3A.0x10`    | `Timing Correction (CRSF Shot)`    | [] |
| `0x3C`         | `Game`                             | [] |
| `0x3E`         | `Reserved`                         | [] |
| `0x40`         | `Reserved`                         | [] |
| `0x78`-`0x79`  | `KISSFC Reserved`                    | [] |
| `0x7A`         | `MSP Request / 0x7B Response`      | [] |
| `0x7F`         | `ArduPilot Legacy Reserved`        | [] |
| `0x80`         | `ArduPilot Reserved Passthrough Frame` | []  |
| `0x81`,`0x82`  | `mLRS Reserved`                    | [] |
| `0xAA`         | `CRSF MAVLink Envelope`            | [] |
| `0xAC`         | `CRSF MAVLink System Status Sensor` | [] |
