#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

#[riot_rs::hw_setup]
mod sensors {}
// mod sensors;

use embassy_time::{Duration, Timer};
use riot_rs::{
    debug::println,
    sensors::{Reading, REGISTRY},
};

#[riot_rs::task(autostart)]
async fn main() {
    loop {
        println!("New measurements:");
        for sensor in REGISTRY.sensors() {
            match riot_rs::sensors::read!(sensor).await {
                Ok(values) => {
                    for (i, value) in values.values().enumerate() {
                        let value_scale = sensor.value_scales().iter().nth(i).unwrap();
                        let unit = sensor.units().iter().nth(i).unwrap();
                        let reading_label = sensor.reading_labels().iter().nth(i).unwrap();
                        let value = value.value() as f32 / 10i32.pow((-value_scale) as u32) as f32;
                        println!(
                            "{} ({}): {} {} ({})",
                            sensor.display_name().unwrap_or("unknown"),
                            sensor.label().unwrap_or("no label"),
                            value,
                            unit,
                            reading_label
                        );
                    }
                }
                Err(err) => {
                    println!("error while reading sensor value: {}", err);
                }
            }
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[cfg(capability = "hw/usb-device-port")]
#[riot_rs::config(usb)]
fn usb_config() -> riot_rs::embassy::embassy_usb::Config<'static> {
    let mut config = riot_rs::embassy::embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("Sensors example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for Windows support.
    config.composite_with_iads = true;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config
}
