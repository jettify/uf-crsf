pub const CRSF_SYNC_BYTE: u8 = 0xC8;
pub const CRSF_MAX_PACKET_SIZE: usize = 64;
// Header (1) + Type (1) + CRC (1) + Payload (min 1)
pub const CRSF_MIN_PACKET_SIZE: usize = 4;
pub const CRSF_PACKET_HEADER_LEN: usize = 2;
