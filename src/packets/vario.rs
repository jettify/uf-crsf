#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VariometerSensor {
    pub v_speed: i16, // Vertical speed cm/s
}

impl VariometerSensor {
    pub const SERIALIZED_LEN: usize = 2;

    pub fn to_bytes(&self, buffer: &mut [u8; Self::SERIALIZED_LEN]) {
        buffer[..2].copy_from_slice(&self.v_speed.to_be_bytes());
    }

    pub fn from_bytes(data: &[u8; Self::SERIALIZED_LEN]) -> Self {
        Self {
            v_speed: i16::from_be_bytes([data[0], data[1]]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vario() {
        let raw_bytes: [u8; 2] = [0; 2];
        let data = &raw_bytes[0..2].try_into().unwrap();

        let vario = VariometerSensor::from_bytes(data);
        assert_eq!(vario.v_speed, 0);

        let mut buffer: [u8; 2] = [0; 2];
        vario.to_bytes(&mut buffer);
        assert_eq!(&buffer, data);
    }
}
