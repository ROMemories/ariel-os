use ariel_os::hal::{peripherals, spi};

#[cfg(context = "esp")]
pub type Spi = spi::main::SPI2;
#[cfg(context = "espressif-esp32-c6-devkitc-1")]
ariel_os::hal::define_peripherals!(Peripherals {
    spi_sck: GPIO0,
    spi_miso: GPIO1,
    spi_mosi: GPIO8,
    spi_cs: GPIO2,
});
