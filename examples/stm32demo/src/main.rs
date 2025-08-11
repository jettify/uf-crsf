#![no_std]
#![no_main]

mod fmt;

#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::usart::{Config as UsartConfig, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart, Config};
use embassy_time::{with_timeout, Duration, Timer};
use fmt::info;
use uf_crsf::parser::ParseResult;
use uf_crsf::CrsfParser;

bind_interrupts!(struct Irqs {
    USART1 => usart::InterruptHandler<peripherals::USART1>;
});

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum ReadError {
    Timeout,
    Uart(usart::Error),
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    let mut usart_config = UsartConfig::default();
    usart_config.baudrate = 420000;

    let crsf_usart = Uart::new(
        p.USART1,
        p.PA10,
        p.PA9,
        Irqs,
        p.DMA2_CH7,
        p.DMA2_CH5,
        usart_config,
    )
    .unwrap();

    let (_tx, rx) = crsf_usart.split();

    const BUFFER_SIZE: usize = 64;
    let mut dma_buf = [0u8; 128];
    let mut buffer = [0u8; BUFFER_SIZE];
    let mut buf_rx = rx.into_ring_buffered(&mut dma_buf);

    let mut parser = CrsfParser::new();
    loop {
        match read_serial_data(&mut buf_rx, &mut buffer).await {
            Ok(bytes) => {
                for &byte in &buffer[..bytes] {
                    match parser.push_byte(byte) {
                        ParseResult::Complete(packet) => info!("{:?}", packet),
                        ParseResult::Error(e) => info!("Parsing error {:?}", e),
                        ParseResult::Incomplete => (),
                    }
                }
            }
            Err(e) => {
                info!("Read error: {:?}", e);
                Timer::after(Duration::from_millis(100)).await;
            }
        }
    }
}

async fn read_serial_data(
    uart_rx: &mut (impl embedded_io_async::Read<Error = usart::Error> + Unpin),
    buffer: &mut [u8],
) -> Result<usize, ReadError> {
    const TIMEOUT: Duration = Duration::from_secs(1);

    with_timeout(TIMEOUT, uart_rx.read(buffer))
        .await
        .map_err(|_| ReadError::Timeout)?
        .map_err(ReadError::Uart)
}

