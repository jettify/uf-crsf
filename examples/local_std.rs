use std::env;
use std::io::ErrorKind;
use std::process::exit;
use std::time::Duration;
use uf_crsf::CrsfParser;

fn main() {
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

    let mut port = serialport::new(&path, 420_000) // Common CRSF baud rate
        .timeout(Duration::from_millis(10))
        .open()
        .unwrap_or_else(|e| {
            eprintln!("Failed to open serial port '{}': {}", &path, e);
            exit(1);
        });

    let mut buf = [0; 1024];
    let mut parser = CrsfParser::new();
    println!("Reading from serial port '{}'...", path);
    loop {
        match port.read(buf.as_mut_slice()) {
            Ok(n) => {
                for packet in parser.iter_packets(&buf[..n]) {
                    println!("{:?}", packet);
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
