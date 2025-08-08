use crate::packets::{CrsfPacket, PacketType};
use crate::CrsfParsingError;

/// Represents a Device Ping packet (0x28).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DevicePing {
    pub dst_addr: u8,
    pub src_addr: u8,
}

impl CrsfPacket for DevicePing {
    const PACKET_TYPE: PacketType = PacketType::DevicePing;
    const MIN_PAYLOAD_SIZE: usize = 2;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        buffer[0] = self.dst_addr;
        buffer[1] = self.src_addr;
        Ok(2)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        Ok(Self {
            dst_addr: data[0],
            src_addr: data[1],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parameter_ping_to_bytes() {
        let ping = DevicePing {
            dst_addr: 0xEA,
            src_addr: 0xEE,
        };
        let mut buffer = [0u8; 2];
        let len = ping.to_bytes(&mut buffer).unwrap();
        assert_eq!(len, 2);
        assert_eq!(buffer, [0xEA, 0xEE]);
    }

    #[test]
    fn test_parameter_ping_from_bytes() {
        let data: [u8; 2] = [0xEA, 0xEE];
        let ping = DevicePing::from_bytes(&data).unwrap();
        assert_eq!(
            ping,
            DevicePing {
                dst_addr: 0xEA,
                src_addr: 0xEE
            }
        );
    }

    #[test]
    fn test_parameter_ping_from_bytes_with_payload() {
        // Should ignore extra payload
        let data: [u8; 5] = [0xEA, 0xEE, 3, 4, 5];
        let ping = DevicePing::from_bytes(&data).unwrap();
        assert_eq!(
            ping,
            DevicePing {
                dst_addr: 0xEA,
                src_addr: 0xEE
            }
        );
    }
}
