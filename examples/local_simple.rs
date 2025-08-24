use uf_crsf::CrsfParser;

fn main() {
    let mut parser = CrsfParser::new();

    // A sample CRSF packet payload for RC channels
    let buf: [u8; 26] = [
        0xC8, // Address
        0x18, // Length
        0x16, // Type (RC Channels)
        0x03, 0x1F, 0x58, 0xC0, 0x07, 0x16, 0xB0, 0x80, 0x05, 0x2C, 0x60, 0x01, 0x0B, 0xF8, 0xC0,
        0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 252,  // Packet
        0x42, // Crc
    ];

    for item in parser.iter_packets(&buf) {
        match item {
            Ok(p) => println!("{:?}", p),
            Err(e) => eprintln!("Error parsing packet: {:?}", e),
        }
    }
}
