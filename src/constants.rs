// per specification, max packet size with all framing
pub const CRSF_MAX_PACKET_SIZE: usize = 64;
// header (1) + packet type (1) + CRC (1) + Payload (min 1)
pub const CRSF_MIN_PACKET_SIZE: usize = 4;
