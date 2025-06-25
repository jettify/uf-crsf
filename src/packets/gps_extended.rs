//    uint8_t fix_type;       // Current GPS fix quality
//    int16_t n_speed;        // Northward (north = positive) Speed [cm/sec]
//    int16_t e_speed;        // Eastward (east = positive) Speed [cm/sec]
//    int16_t v_speed;        // Vertical (up = positive) Speed [cm/sec]
//    int16_t h_speed_acc;    // Horizontal Speed accuracy cm/sec
//    int16_t track_acc;      // Heading accuracy in degrees scaled with 1e-1 degrees times 10)
//    int16_t alt_ellipsoid;  // Meters Height above GPS Ellipsoid (not MSL)
//    int16_t h_acc;          // horizontal accuracy in cm
//    int16_t v_acc;          // vertical accuracy in cm
//    uint8_t reserved;
//    uint8_t hDOP;           // Horizontal dilution of precision,Dimensionless in nits of.1.
//    uint8_t vDOP;           // vertical dilution of precision, Dimensionless in nits of .1.


#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GpsExtended {
    pub fix_type: u8,
    pub n_speed: i16,
    pub e_speed: i16,
    pub v_speed: i16,
    pub h_speed_acc: i16,
    pub track_acc: i16,
    pub alt_ellipsoid: i16,
    pub h_acc: i16,
    pub v_acc: i16,
    pub reserved: u8,
    pub h_dop: u8,
    pub v_dop: u8,
}
