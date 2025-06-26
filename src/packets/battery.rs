//    int16_t voltage;        // Voltage (LSB = 10 µV)
//    int16_t current;        // Current (LSB = 10 µA)
//    uint24_t capacity_used; // Capacity used (mAh)
//    uint8_t remaining;      // Battery remaining (percent)



#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Battery {
    pub voltage: i16,
    pub current: i16,
    pub capacity_used: u24
    pub remaining: u8,
}
