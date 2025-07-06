use crate::error::CrsfParsingError;
use crate::parser::RawCrsfPacket;

mod airspeed;
mod baro_altitude;
mod battery;
mod flight_mode;
mod gps;
mod gps_extended;
mod gps_time;
mod heartbeat;
mod link_statistics;
mod link_statistics_rx;
mod link_statistics_tx;
mod rc_channels_packed;
mod rpm;
mod temp;
mod vario;
mod voltages;
mod vtx_telemetry;

pub use airspeed::AirSpeed;
pub use baro_altitude::BaroAltitude;
pub use battery::Battery;
pub use flight_mode::FlightMode;
pub use gps::Gps;
pub use gps_extended::GpsExtended;
pub use gps_time::GpsTime;
pub use heartbeat::Heartbeat;
pub use link_statistics::LinkStatistics;
pub use link_statistics_rx::LinkStatisticsRx;
pub use link_statistics_tx::LinkStatisticsTx;
pub use rc_channels_packed::RcChannelsPacked;
pub use rpm::Rpm;
pub use temp::Temp;
pub use vario::VariometerSensor;
pub use voltages::Voltages;
pub use vtx_telemetry::VtxTelemetry;

use num_enum::TryFromPrimitive;

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
    NotImlemented(PacketType, usize),
}

impl Packet {
    pub fn parse(raw_packet: &RawCrsfPacket<'_>) -> Result<Packet, CrsfParsingError> {
        let packet_type = PacketType::try_from_primitive(raw_packet.raw_packet_type())
            .map_err(|_| CrsfParsingError::UnexpectedPacketType(raw_packet.raw_packet_type()))?;

        let data = raw_packet.payload().try_into().unwrap();
        match packet_type {
            PacketType::LinkStatistics => {
                Ok(Self::LinkStatistics(LinkStatistics::from_bytes(data)?))
            }
            PacketType::LinkStatisticsTx => {
                Ok(Self::LinkStatisticsTx(LinkStatisticsTx::from_bytes(data)?))
            }
            PacketType::LinkStatisticsRx => {
                Ok(Self::LinkStatisticsRx(LinkStatisticsRx::from_bytes(data)?))
            }
            PacketType::RcChannelsPacked => {
                Ok(Self::RCChannels(RcChannelsPacked::from_bytes(data)?))
            }
            PacketType::Gps => Ok(Self::Gps(Gps::from_bytes(data)?)),
            PacketType::GpsTime => Ok(Self::GpsTime(GpsTime::from_bytes(data)?)),
            PacketType::GpsExtended => Ok(Self::GpsExtended(GpsExtended::from_bytes(data)?)),
            PacketType::AirSpeed => Ok(Self::AirSpeed(AirSpeed::from_bytes(data)?)),
            PacketType::BaroAltitude => Ok(Self::BaroAltitude(BaroAltitude::from_bytes(data)?)),

            PacketType::BatterySensor => Ok(Self::Battery(Battery::from_bytes(data)?)),
            PacketType::FlightMode => Ok(Self::FlightMode(FlightMode::from_bytes(data)?)),
            PacketType::Rpm => Ok(Self::Rpm(Rpm::from_bytes(data)?)),
            PacketType::Temp => Ok(Self::Temp(Temp::from_bytes(data)?)),
            PacketType::Voltages => Ok(Self::Voltages(Voltages::from_bytes(data)?)),
            PacketType::VtxTelemetry => Ok(Self::VtxTelemetry(VtxTelemetry::from_bytes(data)?)),
            PacketType::Vario => Ok(Self::Vario(VariometerSensor::from_bytes(data)?)),
            PacketType::Heartbeat => Ok(Self::Heartbeat(Heartbeat::from_bytes(data)?)),

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
    FlightMode = 0x21,
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

/// A trait representing a deserializable CRSF packet.
pub trait CrsfPacket: Sized {
    /// The CRSF frame type identifier for this packet.
    const FRAME_TYPE: u8;

    /// The minimum expected length of the packet's payload in bytes.
    /// For fixed-size packets, this is the same as the payload size.
    const MIN_PAYLOAD_SIZE: usize;

    /// Creates a packet instance from a payload byte slice.
    /// The slice is guaranteed to have a length of at least `MIN_PAYLOAD_SIZE`.
    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError>;
    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError>;
}
