#![deny(clippy::pedantic)]

#[expect(clippy::missing_panics_doc)]
pub fn init(peripherals: &mut crate::hal::OptionalPeripherals) {
    #[cfg(context = "nrf52840dk")]
    let uart = {
        let uart_rx = peripherals.P0_08.take().unwrap();
        let uart_tx = peripherals.P0_06.take().unwrap();

        embassy_nrf::bind_interrupts!(struct Irqs {
            UARTE0 => embassy_nrf::uarte::InterruptHandler<embassy_nrf::peripherals::UARTE0>;
        });

        embassy_nrf::uarte::Uarte::new(
            peripherals.UARTE0.take().unwrap(),
            Irqs,
            uart_rx,
            uart_tx,
            // &mut rx_buf,
            // &mut tx_buf,
            // config,
            embassy_nrf::uarte::Config::default(),
        )
    };

    // FIXME: should be st-b-l072z-lrwan1
    #[cfg(context = "st-nucleo-h755zi-q")]
    let uart = {
        let uart_rx = peripherals.PA3.take().unwrap();
        let uart_tx = peripherals.PA2.take().unwrap();

        embassy_stm32::usart::Uart::new_blocking(
            peripherals.USART2.take().unwrap(),
            uart_rx,
            uart_tx,
            embassy_stm32::usart::Config::default(),
        )
        .unwrap()
    };

    let _ = ariel_os_debug::backend::DEBUG_UART.init(embassy_sync::mutex::Mutex::new(uart));
}
