use crate::packets::CrsfPacket;
use crate::packets::PacketType;
use crate::CrsfParsingError;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VariometerSensor {
    pub v_speed: i16, // Vertical speed cm/s
}

impl CrsfPacket for VariometerSensor {
    const PACKET_TYPE: PacketType = PacketType::Vario;
    const MIN_PAYLOAD_SIZE: usize = 2;

    fn to_bytes(&self, buffer: &mut [u8]) -> Result<usize, CrsfParsingError> {
        self.validate_buffer_size(buffer)?;
        buffer[..2].copy_from_slice(&self.v_speed.to_be_bytes());
        Ok(Self::MIN_PAYLOAD_SIZE)
    }

    fn from_bytes(data: &[u8]) -> Result<Self, CrsfParsingError> {
        if data.len() != Self::MIN_PAYLOAD_SIZE {
            return Err(CrsfParsingError::InvalidPayloadLength);
        }

        Ok(Self {
            v_speed: i16::from_be_bytes([data[0], data[1]]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vario() {
        let raw_bytes: [u8; 2] = [0; 2];
        let data = &raw_bytes[0..2];

        let vario = VariometerSensor::from_bytes(data).unwrap();
        assert_eq!(vario.v_speed, 0);

        let mut buffer: [u8; 2] = [0; 2];
        vario.to_bytes(&mut buffer).unwrap();
        assert_eq!(&buffer, data);
    }
}
