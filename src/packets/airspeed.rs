//!    uint16_t speed;             // Airspeed in 0.1 * km/h (hectometers/h)
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AirSpeed {
    pub speed: u16,
}


impl AirSpeed {
    pub const SERIALIZED_LEN: usize = 2;

    pub fn to_bytes(&self, buffer: &mut [u8; Self::SERIALIZED_LEN]) {
        buffer[0..2].copy_from_slice(&self.speed.to_be_bytes());
    }

    pub fn from_bytes(data: &[u8; Self::SERIALIZED_LEN]) -> Self {
        Self {
            speed: u16::from_be_bytes(data[0..2].try_into().unwrap()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_airspeed_to_bytes() {
        let airspeed = AirSpeed { speed: 1234 };
        let mut buffer = [0u8; AirSpeed::SERIALIZED_LEN];
        airspeed.to_bytes(&mut buffer);
        assert_eq!(buffer, [0x04, 0xD2]);
    }

    #[test]
    fn test_airspeed_from_bytes() {
        let data: [u8; AirSpeed::SERIALIZED_LEN] = [0x04, 0xD2];
        let airspeed = AirSpeed::from_bytes(&data);
        assert_eq!(airspeed.speed, 1234);
    }

    #[test]
    fn test_airspeed_round_trip() {
        let airspeed = AirSpeed { speed: 5678 };
        let mut buffer = [0u8; AirSpeed::SERIALIZED_LEN];
        airspeed.to_bytes(&mut buffer);
        let round_trip_airspeed = AirSpeed::from_bytes(&buffer);
        assert_eq!(airspeed, round_trip_airspeed);
    }
}
