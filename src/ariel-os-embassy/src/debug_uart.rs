#![deny(clippy::pedantic)]

use crate::hal;

pub static DEBUG_UART: embassy_sync::once_lock::OnceLock<
    embassy_sync::mutex::Mutex<
        embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
        hal::uart::Uart,
    >,
> = embassy_sync::once_lock::OnceLock::new();

#[expect(clippy::missing_panics_doc)]
pub fn init(peripherals: &mut hal::OptionalPeripherals) {
    #[cfg(context = "nrf52840dk")]
    pub type DebugUart<'a> = hal::uart::UARTE0<'a>;
    #[cfg(context = "nrf52840dk")]
    let (uart_rx, uart_tx) = {
        let uart_rx = peripherals.P0_08.take().unwrap();
        let uart_tx = peripherals.P0_06.take().unwrap();

        (uart_rx, uart_tx)
    };

    // FIXME: should be st-b-l072z-lrwan1
    #[cfg(context = "st-nucleo-h755zi-q")]
    pub type DebugUart<'a> = hal::uart::USART2<'a>;
    let (uart_rx, uart_tx) = {
        let uart_rx = peripherals.PA3.take().unwrap();
        let uart_tx = peripherals.PA2.take().unwrap();

        (uart_rx, uart_tx)
    };

    let mut config = hal::uart::Config::default();
    config.baudrate = 115_200;

    // TODO: make the RX buffer much smaller
    static RX_BUF: static_cell::ConstStaticCell<[u8; 32]> =
        static_cell::ConstStaticCell::new([0; 32]);
    static TX_BUF: static_cell::ConstStaticCell<[u8; 32]> =
        static_cell::ConstStaticCell::new([0; 32]);

    let uart = DebugUart::new(uart_rx, uart_tx, RX_BUF.take(), TX_BUF.take(), config);

    let _ = DEBUG_UART.init(embassy_sync::mutex::Mutex::new(uart));
}

// SAFETY: the compiler prevents from defining multiple functions with the same name in the same
// crate.
#[unsafe(no_mangle)]
fn __ariel_os_debug_uart_write(buffer: &[u8]) {
    use embedded_io_async::Write;
    // use embedded_io::Write;

    // FIXME: do not unwrap
    embassy_futures::block_on(async {
        // This effectively drops any debug output until the UART driver is populated.
        // If we instead waited on it to be set, this would deadlock when trying to print
        // on the debug output before the driver is populated.
        if let Some(uart) = DEBUG_UART.try_get() {
            let mut uart = uart.lock().await;
            uart.write(buffer).await.unwrap();
            // uart.write(buffer).unwrap();
            // TODO: is flushing needed here?
            uart.flush().await.unwrap();
            // uart.flush().unwrap();
        }
    });
}
