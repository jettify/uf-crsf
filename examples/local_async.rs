use embedded_io_adapters::tokio_1::FromTokio;
use std::env;
use std::process::exit;
use std::time::Duration;

use tokio_serial::SerialPortBuilderExt;
use uf_crsf::CrsfParser;

#[tokio::main]
async fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        println!("No serial port specified. Usage: local_async <PATH>");

        let ports = match serialport::available_ports() {
            Ok(ports) => ports,
            Err(e) => {
                eprintln!("Failed to enumerate serial ports: {}", e);
                exit(1);
            }
        };
        println!("Available ports:");
        for p in &ports {
            println!("  {}", p.port_name);
        }
        exit(1);
    });

    let mut port = tokio_serial::new(&path, 420_000)
        .timeout(Duration::from_millis(10))
        .open_native_async()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open serial port '{}': {}", &path, e);
            exit(1);
        });

    let mut adapted_port = FromTokio::new(&mut port);
    let mut parser = CrsfParser::new();
    println!("Reading from serial port '{}'...", path);

    loop {
        match parser.read_packet(&mut adapted_port).await {
            Ok(packet) => {
                println!("{:?}", packet);
            }
            Err(e) => {
                eprintln!("Error reading from serial port: {:?}", e);
                break;
            }
        }
    }
}
