use crate::constants;
use crate::error::CrsfParsingError;
use crate::parser::RawCrsfPacket;
use crc;

mod airspeed;
mod baro_altitude;
mod battery;
mod esp_now;
mod flight_mode;
mod gps;
mod gps_extended;
mod gps_time;
mod heartbeat;
mod link_statistics;
mod link_statistics_rx;
mod link_statistics_tx;
mod mavlink_envelope;
mod mavlink_fc;
mod rc_channels_packed;
mod remote;
mod rpm;
mod temp;
mod vario;
mod voltages;
mod vtx_telemetry;

pub use airspeed::AirSpeed;
pub use baro_altitude::BaroAltitude;
pub use battery::Battery;
pub use esp_now::EspNow;
pub use flight_mode::FlightMode;
pub use gps::Gps;
pub use gps_extended::GpsExtended;
pub use gps_time::GpsTime;
pub use heartbeat::Heartbeat;
pub use link_statistics::LinkStatistics;
pub use link_statistics_rx::LinkStatisticsRx;
pub use link_statistics_tx::LinkStatisticsTx;
pub use mavlink_envelope::MavlinkEnvelope;
pub use mavlink_fc::MavLinkFc;
pub use rc_channels_packed::RcChannelsPacked;
pub use remote::Remote;
pub use rpm::Rpm;
pub use temp::Temp;
pub use vario::VariometerSensor;
pub use voltages::Voltages;
pub use vtx_telemetry::VtxTelemetry;

use num_enum::TryFromPrimitive;

/// A trait representing a deserializable CRSF packet.
pub trait CrsfPacket: Sized {
    /// The CRSF frame type identifier for this packet.
    const PACKET_TYPE: PacketType;

    /// The minimum expected length of the packet's payload in bytes.
    /// For fixed-size packets, this is the same as the payload size.
    const MIN_PAYLOAD_SIZE: usize;

    /// Creates a packet instance from a payload byte slice.
    /// The slice is guaranteed to have a length of at least `MIN_PAYLOAD_SIZE`.
    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError>;
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError>;

    fn validate_buffer_size(&self, buffer: &[u8]) -> Result<(), CrsfParsingError> {
        if buffer.len() < Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::BufferOverflow);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Packet {
    LinkStatistics(LinkStatistics),
    LinkStatisticsRx(LinkStatisticsRx),
    LinkStatisticsTx(LinkStatisticsTx),
    RCChannels(RcChannelsPacked),
    Gps(Gps),
    GpsTime(GpsTime),
    GpsExtended(GpsExtended),
    Vario(VariometerSensor),
    Battery(Battery),
    AirSpeed(AirSpeed),
    BaroAltitude(BaroAltitude),
    Rpm(Rpm),
    Temp(Temp),
    Voltages(Voltages),
    VtxTelemetry(VtxTelemetry),
    FlightMode(FlightMode),
    Heartbeat(Heartbeat),
    EspNow(EspNow),
    MavlinkEnvelope(MavlinkEnvelope),
    MavLinkFc(MavLinkFc),
    Remote(Remote),
    NotImlemented(PacketType, usize),
}

impl Packet {
    pub fn parse(raw_packet: &RawCrsfPacket<'_>) -> Result<Packet, CrsfParsingError> {
        let packet_type = PacketType::try_from_primitive(raw_packet.raw_packet_type())
            .map_err(|_| CrsfParsingError::UnexpectedPacketType(raw_packet.raw_packet_type()))?;

        let data = raw_packet.payload();
        match packet_type {
            LinkStatistics::PACKET_TYPE => {
                Ok(Self::LinkStatistics(LinkStatistics::from_bytes(data)?))
            }
            LinkStatisticsTx::PACKET_TYPE => {
                Ok(Self::LinkStatisticsTx(LinkStatisticsTx::from_bytes(data)?))
            }
            LinkStatisticsRx::PACKET_TYPE => {
                Ok(Self::LinkStatisticsRx(LinkStatisticsRx::from_bytes(data)?))
            }
            RcChannelsPacked::PACKET_TYPE => {
                Ok(Self::RCChannels(RcChannelsPacked::from_bytes(data)?))
            }
            Gps::PACKET_TYPE => Ok(Self::Gps(Gps::from_bytes(data)?)),
            GpsTime::PACKET_TYPE => Ok(Self::GpsTime(GpsTime::from_bytes(data)?)),
            GpsExtended::PACKET_TYPE => Ok(Self::GpsExtended(GpsExtended::from_bytes(data)?)),
            AirSpeed::PACKET_TYPE => Ok(Self::AirSpeed(AirSpeed::from_bytes(data)?)),
            BaroAltitude::PACKET_TYPE => Ok(Self::BaroAltitude(BaroAltitude::from_bytes(data)?)),
            Battery::PACKET_TYPE => Ok(Self::Battery(Battery::from_bytes(data)?)),
            FlightMode::PACKET_TYPE => Ok(Self::FlightMode(FlightMode::from_bytes(data)?)),
            Rpm::PACKET_TYPE => Ok(Self::Rpm(Rpm::from_bytes(data)?)),
            Temp::PACKET_TYPE => Ok(Self::Temp(Temp::from_bytes(data)?)),
            Voltages::PACKET_TYPE => Ok(Self::Voltages(Voltages::from_bytes(data)?)),
            VtxTelemetry::PACKET_TYPE => Ok(Self::VtxTelemetry(VtxTelemetry::from_bytes(data)?)),
            VariometerSensor::PACKET_TYPE => Ok(Self::Vario(VariometerSensor::from_bytes(data)?)),
            EspNow::PACKET_TYPE => Ok(Self::EspNow(EspNow::from_bytes(data)?)),
            Heartbeat::PACKET_TYPE => Ok(Self::Heartbeat(Heartbeat::from_bytes(data)?)),
            MavLinkFc::PACKET_TYPE => Ok(Self::MavLinkFc(MavLinkFc::from_bytes(data)?)),
            MavlinkEnvelope::PACKET_TYPE => {
                Ok(Self::MavlinkEnvelope(MavlinkEnvelope::from_bytes(data)?))
            }
            _ => Ok(Packet::NotImlemented(
                packet_type,
                raw_packet.payload().len(),
            )),
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, num_enum::TryFromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PacketType {
    Gps = 0x02,
    GpsTime = 0x03,
    GpsExtended = 0x06,
    Vario = 0x07,
    BatterySensor = 0x08,
    BaroAltitude = 0x09,
    AirSpeed = 0x0A,
    Rpm = 0x0C,
    Temp = 0x0D,
    Voltages = 0x0E,
    VtxTelemetry = 0x10,
    Heartbeat = 0x0B,
    LinkStatistics = 0x14,
    RcChannelsPacked = 0x16,
    SubsetRcChannelsPacked = 0x17,
    LinkStatisticsRx = 0x1C,
    LinkStatisticsTx = 0x1D,
    Attitude = 0x1E,
    MavLinkFc = 0x1F,
    FlightMode = 0x21,
    EspNow = 0x22,
    DevicePing = 0x28,
    DeviceInfo = 0x29,
    ParameterSettingsEntry = 0x2B,
    ParameterRead = 0x2C,
    ParameterWrite = 0x2D,
    ElrsStatus = 0x2E,
    Command = 0x32,
    RadioId = 0x3A,
    KissRequest = 0x78,
    KissResponse = 0x79,
    MspRequest = 0x7A,
    MspResponse = 0x7B,
    MspWrite = 0x7C,
    ArdupilotResponse = 0x80,
    MavlinkEnvelope = 0xAA,
}

impl PacketType {
    pub fn is_extended(self) -> bool {
        self as u8 >= 0x28
    }
}

/// Represents all CRSF packet addresses
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, TryFromPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PacketAddress {
    Broadcast = 0x00,
    Cloud = 0x0E,
    Usb = 0x10,
    Bluetooth = 0x12,
    WifiReceiver = 0x13,
    VideoReceiver = 0x14,
    TbsCorePnpPro = 0x80,
    Esc1 = 0x90,
    Esc2 = 0x91,
    Esc3 = 0x92,
    Esc4 = 0x93,
    Esc5 = 0x94,
    Esc6 = 0x95,
    Esc7 = 0x96,
    Esc8 = 0x97,
    CurrentSensor = 0xC0,
    Gps = 0xC2,
    TbsBlackbox = 0xC4,
    FlightController = 0xC8,
    RaceTag = 0xCC,
    VTX = 0xCE,
    Handset = 0xEA,
    Receiver = 0xEC,
    Transmitter = 0xEE,
}

pub fn write_packet_to_buffer<T: CrsfPacket>(
    buffer: &mut [u8],
    dest: PacketAddress,
    packet: &T,
) -> Result<usize, CrsfParsingError> {
    const MAX_PAYLOAD_SIZE: usize = constants::CRSF_MAX_PACKET_SIZE - 4;
    let mut payload_buf = [0u8; MAX_PAYLOAD_SIZE];

    let payload_size = packet.to_bytes(&mut payload_buf)?;

    let total_frame_size = payload_size + 4;
    if buffer.len() < total_frame_size {
        return Err(CrsfParsingError::BufferOverflow);
    }

    // length byte = 2 (type + crc) + payload_size
    let length_byte = (payload_size + 2) as u8;

    buffer[0] = dest as u8;
    buffer[1] = length_byte;
    buffer[2] = T::PACKET_TYPE as u8;
    buffer[3..3 + payload_size].copy_from_slice(&payload_buf[..payload_size]);

    // CRC is calculated over type and payload
    let crc_payload = &buffer[2..3 + payload_size];
    let crc8_dvb_s2 = crc::Crc::<u8>::new(&crc::CRC_8_DVB_S2);
    let mut digest = crc8_dvb_s2.digest();
    digest.update(crc_payload);
    let calculated_crc = digest.finalize();

    buffer[3 + payload_size] = calculated_crc;

    Ok(total_frame_size)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPacket {
        payload: [u8; 2],
    }

    impl CrsfPacket for MockPacket {
        const PACKET_TYPE: PacketType = PacketType::Command;
        const MIN_PAYLOAD_SIZE: usize = 2;

        fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
            // Not needed for this test
            Ok(Self {
                payload: [data[0], data[1]],
            })
        }

        fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
            buffer[..2].copy_from_slice(&self.payload);
            Ok(2)
        }
    }

    #[test]
    fn test_write_packet_to_buffer() {
        let mut buffer = [0u8; 64];
        let packet = MockPacket {
            payload: [0xAA, 0xBB],
        };
        let dest = PacketAddress::FlightController;

        let result = write_packet_to_buffer(&mut buffer, dest, &packet);

        assert!(result.is_ok());
        let bytes_written = result.unwrap();
        assert_eq!(bytes_written, 6);

        // Destination, Length, Type, Payload, CRC
        // Length = Type (1) + Payload (2) + CRC (1) = 4
        // CRC is calculated over Type and Payload
        let expected_packet: [u8; 6] = [
            dest as u8,
            4,
            MockPacket::PACKET_TYPE as u8,
            0xAA,
            0xBB,
            0x32,
        ];
        assert_eq!(&buffer[..bytes_written], &expected_packet[..]);
    }

    #[test]
    fn test_write_packet_to_buffer_overflow() {
        let mut buffer = [0u8; 5]; // Too small for a 6-byte packet
        let packet = MockPacket {
            payload: [0xAA, 0xBB],
        };
        let dest = PacketAddress::FlightController;

        let result = write_packet_to_buffer(&mut buffer, dest, &packet);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CrsfParsingError::BufferOverflow);
    }
}
