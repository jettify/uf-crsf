use embedded_io_adapters::tokio_1::FromTokio;
use std::env;
use std::io::ErrorKind;
use std::process::exit;
use std::time::Duration;

use tokio_serial::SerialPortBuilderExt;
use uf_crsf::async_io::AsyncCrsfReader;

#[tokio::main]
async fn main() {
    let ports = match serialport::available_ports() {
        Ok(ports) => ports,
        Err(e) => {
            eprintln!("Failed to enumerate serial ports: {}", e);
            exit(1);
        }
    };

    if ports.is_empty() {
        eprintln!("No serial ports found.");
        eprintln!("Please specify a serial port path as an argument.");
        exit(1);
    }

    let path = env::args().nth(1).unwrap_or_else(|| {
        const DEFAULT_PORT: &str = "/dev/tty.usbmodem00000000001B1";
        if ports.iter().any(|p| p.port_name == DEFAULT_PORT) {
            println!(
                "No serial port specified. Using default port: {}",
                DEFAULT_PORT
            );
            DEFAULT_PORT.to_string()
        } else {
            println!("No serial port specified. Available ports:");
            for p in &ports {
                println!("  {}", p.port_name);
            }
            println!("\nUsing first available port: {}", ports[0].port_name);
            ports[0].port_name.clone()
        }
    });

    let mut port = tokio_serial::new(&path, 420_000)
        .timeout(Duration::from_millis(10))
        .open_native_async()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open serial port '{}': {}", &path, e);
            exit(1);
        });

    let adapted_port = FromTokio::new(&mut port);
    let mut reader = AsyncCrsfReader::new(adapted_port);
    println!("Reading from serial port '{}'...", path);

    loop {
        match reader.read_packet().await {
            Ok(packet) => {
                println!("{:?}", packet);
            }
            Err(e) => {
                eprintln!("Error reading from serial port: {:?}", e);
            }
        }
    }
}
