#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]
#![feature(used_with_arg)]

mod buses;
mod pins;
mod sensors;

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
            if let Err(err) = sensor.trigger_measurement() {
                println!(
                    "error while triggering measurement for {}: {}",
                    sensor.display_name().unwrap(),
                    err
                );
                continue;
            }
        }

        for sensor in REGISTRY.sensors() {
            match sensor.wait_for_reading().await {
                Ok(values) => {
                    for (value, reading_axis) in values.values().zip(sensor.reading_axes().iter()) {
                        // FIXME: print accuracy
                        let value = value.value() as f32
                            / 10i32.pow((-reading_axis.scaling()) as u32) as f32;
                        println!(
                            "{} ({}): {} {} ({})",
                            sensor.display_name().unwrap_or("unknown"),
                            sensor.label().unwrap_or("no label"),
                            value,
                            reading_axis.unit(),
                            reading_axis.label(),
                        );
                    }
                }
                Err(err) => {
                    println!(
                        "error while reading sensor value from {}: {}",
                        sensor.display_name().unwrap(),
                        err
                    );
                }
            }
        }

        Timer::after(Duration::from_millis(1000)).await;
    }
}
