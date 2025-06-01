use std::time::Duration;
use std::{env, io};

fn main() {
    let ports = serialport::available_ports().expect("No ports found.");
    println!("{:#?}", ports);

    let path = env::args()
        .nth(1)
        .unwrap_or("/dev/tty.usbmodem00000000001B1".to_owned());
    let mut port = serialport::new(&path, 115_200)
        .timeout(Duration::from_millis(2000))
        .open()
        .expect("Can not open port.");

    let mut buf = [0; 1024];
    loop {
        match port.read(buf.as_mut_slice()) {
            Ok(n @ 1..) => println!("{:?}", &buf[..n]),
            Ok(0) => eprintln!("EOF"),
            Err(err) if err.kind() == std::io::ErrorKind::TimedOut => eprint!("TimedOut"),
            Err(err) => println!("{:?}", err),
        };
    }
}
