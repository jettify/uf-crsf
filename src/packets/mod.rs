mod airspeed;
mod baro_altitude;
mod battery;
mod gps;
mod gps_extended;
mod gps_time;
mod heartbeat;
mod link_statistics;
mod link_statistics_rx;
mod rc_channels_packed;
mod vario;
mod vtx_telemetry;

pub use airspeed::AirSpeed;
pub use battery::Battery;
pub use gps::Gps;
pub use gps_extended::GpsExtended;
pub use gps_time::GpsTime;
pub use link_statistics::LinkStatistics;
pub use rc_channels_packed::RcChannelsPacked;
pub use vario::VariometerSensor;

use num_enum::TryFromPrimitive;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawCrsfPacket<'a> {
    bytes: &'a [u8],
}

impl<'a> RawCrsfPacket<'a> {
    pub fn new(bytes: &'a [u8]) -> Option<Self> {
        if bytes.len() >= 4 {
            Some(Self { bytes })
        } else {
            None
        }
    }

    pub fn dst_addr(&self) -> u8 {
        self.bytes[0]
    }
    pub fn raw_packet_type(&self) -> u8 {
        // XXX
        self.bytes[2]
    }

    pub fn payload(&self) -> &[u8] {
        // XXX
        &self.bytes[3..self.bytes.len() - 1]
    }
    pub fn crc(&self) -> u8 {
        *self.bytes.last().unwrap()
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrsfParsingError {
    UnexpectedPacketType(u8),
    PacketNotImlemented(u8),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Packet {
    LinkStatistics(LinkStatistics),
    RCChannels(RcChannelsPacked),
    Gps(Gps),
    GpsTime(GpsTime),
    GpsExtended(GpsExtended),
    Vario(VariometerSensor),
    Battery(Battery),
    AirSpeed(AirSpeed),
    NotImlemented(PacketType, usize),
}

impl Packet {
    pub fn parse(raw_packet: &RawCrsfPacket<'_>) -> Result<Packet, CrsfParsingError> {
        let packet_type = PacketType::try_from_primitive(raw_packet.raw_packet_type())
            .map_err(|_| CrsfParsingError::UnexpectedPacketType(raw_packet.raw_packet_type()))?;
        match packet_type {
            PacketType::LinkStatistics
                if raw_packet.payload().len() == LinkStatistics::SERIALIZED_LEN =>
            {
                let data = raw_packet.payload().try_into().unwrap();
                Ok(Self::LinkStatistics(LinkStatistics::from_bytes(data)))
            }
            PacketType::RcChannelsPacked
                if raw_packet.payload().len() == RcChannelsPacked::SERIALIZED_LEN =>
            {
                let data = raw_packet.payload().try_into().unwrap();
                Ok(Self::RCChannels(RcChannelsPacked::from_bytes(data)))
            }
            PacketType::Gps if raw_packet.payload().len() == Gps::SERIALIZED_LEN => {
                let data = raw_packet.payload().try_into().unwrap();
                Ok(Self::Gps(Gps::from_bytes(data)))
            }
            PacketType::GpsTime if raw_packet.payload().len() == GpsTime::SERIALIZED_LEN => {
                let data = raw_packet.payload().try_into().unwrap();
                Ok(Self::GpsTime(GpsTime::from_bytes(data)))
            }
            PacketType::GpsExtended
                if raw_packet.payload().len() == GpsExtended::SERIALIZED_LEN =>
            {
                let data = raw_packet.payload().try_into().unwrap();
                Ok(Self::GpsExtended(GpsExtended::from_bytes(data)))
            }

            PacketType::BatterySensor if raw_packet.payload().len() == Battery::SERIALIZED_LEN => {
                let data = raw_packet.payload().try_into().unwrap();
                Ok(Self::Battery(Battery::from_bytes(data)))
            }
            PacketType::AirSpeed if raw_packet.payload().len() == AirSpeed::SERIALIZED_LEN => {
                let data = raw_packet.payload().try_into().unwrap();
                Ok(Self::AirSpeed(AirSpeed::from_bytes(data)))
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
    //
    Heartbeat = 0x0B,
    LinkStatistics = 0x14,
    RcChannelsPacked = 0x16,
    SubsetRcChannelsPacked = 0x17,
    LinkRxId = 0x1C,
    LinkTxId = 0x1D,
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
