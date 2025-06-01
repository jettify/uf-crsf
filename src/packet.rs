#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawCrsfPacket<'a> {
    pub bytes: &'a [u8],
}

impl RawCrsfPacket<'_> {
    pub fn packet_type(&self) -> PacketType {
        PacketType::from_u8(self.bytes[2])
    }

    pub fn payload(&self) -> &[u8] {
        &self.bytes[3..self.bytes.len() - 1]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Packet {
    LinkStatistics(LinkStatistics),
    RCChannels(RcChannelsPacked),
    NotImlemented(PacketType, usize),
}

impl Packet {
    pub fn parse(raw_packet: &RawCrsfPacket<'_>) -> Self {
        match raw_packet.packet_type() {
            PacketType::LinkStatistics
                if raw_packet.payload().len() == LinkStatistics::SERIALIZED_LEN =>
            {
                let data = raw_packet.payload().try_into().unwrap();
                Self::LinkStatistics(LinkStatistics::from_bytes(data))
            }
            PacketType::RcChannelsPacked
                if raw_packet.payload().len() == RcChannelsPacked::SERIALIZED_LEN =>
            {
                let data = raw_packet.payload().try_into().unwrap();
                Self::RCChannels(RcChannelsPacked::from_bytes(data))
            }
            _ => Packet::NotImlemented(raw_packet.packet_type(), raw_packet.payload().len()),
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PacketType {
    Gps = 0x02,
    Vario = 0x07,
    BatterySensor = 0x08,
    BaroAltitude = 0x09,
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
    Unknown,
}

impl PacketType {
    pub fn from_u8(value: u8) -> PacketType {
        match value {
            0x02 => Self::Gps,
            0x07 => Self::Vario,
            0x08 => Self::BatterySensor,
            0x09 => Self::BaroAltitude,
            0x0B => Self::Heartbeat,
            0x14 => Self::LinkStatistics,
            0x16 => Self::RcChannelsPacked,
            0x17 => Self::SubsetRcChannelsPacked,
            0x1C => Self::LinkRxId,
            0x1D => Self::LinkTxId,
            0x1E => Self::Attitude,
            0x21 => Self::FlightMode,
            0x28 => Self::DevicePing,
            0x29 => Self::DeviceInfo,
            0x2B => Self::ParameterSettingsEntry,
            0x2C => Self::ParameterRead,
            0x2D => Self::ParameterWrite,
            0x2E => Self::ElrsStatus,
            0x32 => Self::Command,
            0x3A => Self::RadioId,
            0x78 => Self::KissRequest,
            0x79 => Self::KissResponse,
            0x7A => Self::MspRequest,
            0x7B => Self::MspResponse,
            0x7C => Self::MspWrite,
            0x80 => Self::ArdupilotResponse,
            _ => Self::Unknown,
        }
    }
    pub fn is_extended(self) -> bool {
        self as u8 >= 0x28
    }
}

/// Represents all CRSF packet addresses
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum PacketAddress {
    Broadcast = 0x00,
    Usb = 0x10,
    Bluetooth = 0x12,
    TbsCorePnpPro = 0x80,
    Reserved1 = 0x8A,
    CurrentSensor = 0xC0,
    Gps = 0xC2,
    TbsBlackbox = 0xC4,
    FlightController = 0xC8,
    Reserved2 = 0xCA,
    RaceTag = 0xCC,
    Handset = 0xEA,
    Receiver = 0xEC,
    Transmitter = 0xEE,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[allow(missing_docs)]
pub struct LinkStatistics {
    pub uplink_rssi_1: u8,
    pub uplink_rssi_2: u8,
    pub uplink_link_quality: u8,
    pub uplink_snr: i8,
    pub active_antenna: u8,
    pub rf_mode: u8,
    pub uplink_tx_power: u8,
    pub downlink_rssi: u8,
    pub downlink_link_quality: u8,
    pub downlink_snr: i8,
}

impl LinkStatistics {
    pub const SERIALIZED_LEN: usize = 10;

    pub fn to_bytes(&self, buffer: &mut [u8; Self::SERIALIZED_LEN]) {
        buffer[0] = self.uplink_rssi_1;
        buffer[1] = self.uplink_rssi_2;
        buffer[2] = self.uplink_link_quality;
        buffer[3] = self.uplink_snr as u8;
        buffer[4] = self.active_antenna;
        buffer[5] = self.rf_mode;
        buffer[6] = self.uplink_tx_power;
        buffer[7] = self.downlink_rssi;
        buffer[8] = self.downlink_link_quality;
        buffer[9] = self.downlink_snr as u8;
    }

    pub fn from_bytes(data: &[u8; Self::SERIALIZED_LEN]) -> Self {
        Self {
            uplink_rssi_1: data[0],
            uplink_rssi_2: data[1],
            uplink_link_quality: data[2],
            uplink_snr: data[3] as i8,
            active_antenna: data[4],
            rf_mode: data[5],
            uplink_tx_power: data[6],
            downlink_rssi: data[7],
            downlink_link_quality: data[8],
            downlink_snr: data[9] as i8,
        }
    }
}

/// Represents a RcChannelsPacked packet
#[derive(Debug, PartialEq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RcChannelsPacked(pub [u16; 16]);

impl RcChannelsPacked {
    pub const SERIALIZED_LEN: usize = 22;

    pub fn to_bytes(&self, data: &mut [u8; Self::SERIALIZED_LEN]) {
        let ch = &self.0;
        data[0] = (ch[0]) as u8;
        data[1] = (ch[0] >> 8 | ch[1] << 3) as u8;
        data[2] = (ch[1] >> 5 | ch[2] << 6) as u8;
        data[3] = (ch[2] >> 2) as u8;
        data[4] = (ch[2] >> 10 | ch[3] << 1) as u8;
        data[5] = (ch[3] >> 7 | ch[4] << 4) as u8;
        data[6] = (ch[4] >> 4 | ch[5] << 7) as u8;
        data[7] = (ch[5] >> 1) as u8;
        data[8] = (ch[5] >> 9 | ch[6] << 2) as u8;
        data[9] = (ch[6] >> 6 | ch[7] << 5) as u8;
        data[10] = (ch[7] >> 3) as u8;
        data[11] = (ch[8]) as u8;
        data[12] = (ch[8] >> 8 | ch[9] << 3) as u8;
        data[13] = (ch[9] >> 5 | ch[10] << 6) as u8;
        data[14] = (ch[10] >> 2) as u8;
        data[15] = (ch[10] >> 10 | ch[11] << 1) as u8;
        data[16] = (ch[11] >> 7 | ch[12] << 4) as u8;
        data[17] = (ch[12] >> 4 | ch[13] << 7) as u8;
        data[18] = (ch[13] >> 1) as u8;
        data[19] = (ch[13] >> 9 | ch[14] << 2) as u8;
        data[20] = (ch[14] >> 6 | ch[15] << 5) as u8;
        data[21] = (ch[15] >> 3) as u8;
    }

    pub fn from_bytes(data: &[u8; Self::SERIALIZED_LEN]) -> Self {
        let data: [u16; Self::SERIALIZED_LEN] = core::array::from_fn(|i| data[i] as u16);
        const MASK_11BIT: u16 = 0x07FF;
        let mut ch = [MASK_11BIT; 16];
        ch[0] &= data[0] | (data[1] << 8);
        ch[1] &= (data[1] >> 3) | (data[2] << 5);
        ch[2] &= (data[2] >> 6) | (data[3] << 2) | (data[4] << 10);
        ch[3] &= (data[4] >> 1) | (data[5] << 7);
        ch[4] &= (data[5] >> 4) | (data[6] << 4);
        ch[5] &= (data[6] >> 7) | (data[7] << 1) | (data[8] << 9);
        ch[6] &= (data[8] >> 2) | (data[9] << 6);
        ch[7] &= (data[9] >> 5) | (data[10] << 3);
        ch[8] &= data[11] | (data[12] << 8);
        ch[9] &= (data[12] >> 3) | (data[13] << 5);
        ch[10] &= (data[13] >> 6) | (data[14] << 2) | (data[15] << 10);
        ch[11] &= (data[15] >> 1) | (data[16] << 7);
        ch[12] &= (data[16] >> 4) | (data[17] << 4);
        ch[13] &= (data[17] >> 7) | (data[18] << 1) | (data[19] << 9);
        ch[14] &= (data[19] >> 2) | (data[20] << 6);
        ch[15] &= (data[20] >> 5) | (data[21] << 3);

        RcChannelsPacked(ch)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::LinkStatistics;
    use super::*;
    #[test]
    fn test_basic_link_stat() {
        let raw_bytes: [u8; 14] = [0xC8, 12, 0x14, 16, 19, 99, 151, 1, 2, 3, 8, 88, 148, 252];
        let data = &raw_bytes[3..13].try_into().unwrap();
        let ls = LinkStatistics::from_bytes(data);
        let mut buffer: [u8; 10] = [0; 10];
        ls.to_bytes(&mut buffer);
        assert_eq!(ls.uplink_rssi_1, 16);
        assert_eq!(&buffer, data);
        std::dbg!(ls);
    }

    #[test]
    fn test_rc_channeld() {
        let raw_bytes: [u8; 25] = [
            0xC8, 24, 0x16, 0x03, 0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 0x60, 0x01,
            0x0B, 0xF8, 0xC0, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 252,
        ];
        let data = &raw_bytes[3..25].try_into().unwrap();
        let rc = RcChannelsPacked::from_bytes(data);
        let mut buffer: [u8; 22] = [0; 22];
        rc.to_bytes(&mut buffer);
        assert_eq!(&buffer, data);
        std::dbg!(rc);
    }
}
