#![no_main]
#![no_std]

mod pins;

use ariel_os::{
    debug::log::{defmt, info, error},
    hal,
    i2c::controller::{Kilohertz, highest_freq_in},
};
use embedded_hal_async::i2c::I2c as _;

const TARGET_I2C_ADDR: u8 = 0x10;

#[ariel_os::task(autostart, peripherals)]
async fn blinky(peripherals: pins::LedPeripherals) {
    let mut rx_buffer = [0u8; 255];

    let mut i2c_config = hal::i2c::controller::Config::default();
    i2c_config.frequency = const { highest_freq_in(Kilohertz::kHz(100)..=Kilohertz::kHz(400)) };
    info!("Selected frequency: {}", i2c_config.frequency);

    let mut i2c_bus = pins::SensorI2c::new(peripherals.i2c_sda, peripherals.i2c_scl, i2c_config);

    let mut parser = nmea0183::Parser::new();

    loop {
        i2c_bus.read(TARGET_I2C_ADDR, &mut rx_buffer).await.unwrap();

        let end_index = rx_buffer
            .iter()
            .position(|b| *b == 0x0a)
            .unwrap_or(rx_buffer.len());
        if end_index == 0 {
            continue;
        }
        // FIXME: check that ASCII is valid
        let nmea = str::from_utf8(&rx_buffer[..end_index]).unwrap();

        info!("NMEA: {}", nmea);

        for result in parser.parse_from_bytes(&rx_buffer) {
            match result {
                Ok(nmea0183::ParseResult::RMC(Some(rmc))) => {
                    info!("RMC: {:?}", defmt::Debug2Format(&rmc));
                }, // Got RMC sentence
                Ok(nmea0183::ParseResult::GGA(Some(gga))) => {
                    info!("GGA: {:?}", defmt::Debug2Format(&gga));
                }, // Got GGA sentence without valid data, receiver ok but has no solution
                Ok(other) => {
                    info!("Other: {:?}", defmt::Debug2Format(&other));
                }, // Some other sentences..
                Err(err) => {
                    error!("Error: {:?}", err);
                } // Got parse error
            }
        }
    }
}
