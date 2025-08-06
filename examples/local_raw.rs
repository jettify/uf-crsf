use std::io::{ErrorKind, Read};
use std::process::exit;
use std::time::Duration;
use uf_crsf::packets::PacketType;
use uf_crsf::parser::ParseResult;
use uf_crsf::CrsfParser;

fn main() {
    let port_name: &str = "/dev/tty.usbmodem00000000001B1";
    println!("Using serial port: {}", port_name);

    let mut port = serialport::new(port_name, 420_000)
        .timeout(Duration::from_millis(10))
        .open()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open serial port '{}': {}", port_name, e);
            exit(1);
        });

    let mut buf = [0; 1024];
    let mut parser = CrsfParser::new();
    println!("Reading from serial port '{}'...", port_name);
    loop {
        match port.read(buf.as_mut_slice()) {
            Ok(n) => {
                for &byte in &buf[..n] {
                    let raw_packet_result = parser.push_byte_raw(byte);
                    match raw_packet_result {
                        ParseResult::Complete(raw_packet) => {
                            let packet_type = PacketType::try_from(raw_packet.raw_packet_type());
                            println!("{:?} -> {:?}", packet_type, raw_packet);
                        }
                        ParseResult::Error(e) => println!("Parsing error: {:?}", e),
                        ParseResult::Incomplete => (),
                    }
                }
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                // This is expected when no data is coming in
            }
            Err(e) => {
                eprintln!("Error reading from serial port: {}", e);
                break;
            }
        }
    }
}
