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
    gpio::{Level, Output},
    sensors::{Reading, REGISTRY},
};

// #[riot_rs::task(autostart)]
// async fn main() {
// loop {
//     println!("New measurements:");
//     for sensor in REGISTRY.sensors() {
//         if let Err(err) = sensor.trigger_measurement() {
//             println!(
//                 "error while triggering measurement for {}: {}",
//                 sensor.display_name().unwrap(),
//                 err
//             );
//             continue;
//         }
//     }
//
//     for sensor in REGISTRY.sensors() {
//         match sensor.wait_for_reading().await {
//             Ok(values) => {
//                 for (value, reading_axis) in values.values().zip(sensor.reading_axes().iter()) {
//                     // FIXME: print accuracy
//                     let value = value.value() as f32
//                         / 10i32.pow((-reading_axis.scaling()) as u32) as f32;
//                     println!(
//                         "{} ({}): {} {} ({})",
//                         sensor.display_name().unwrap_or("unknown"),
//                         sensor.label().unwrap_or("no label"),
//                         value,
//                         reading_axis.unit(),
//                         reading_axis.label(),
//                     );
//                 }
//             }
//             Err(err) => {
//                 println!(
//                     "error while reading sensor value from {}: {}",
//                     sensor.display_name().unwrap(),
//                     err
//                 );
//             }
//         }
//     }
//
//     Timer::after(Duration::from_millis(1000)).await;
// }
// }

#[riot_rs::task(autostart, peripherals)]
async fn accel_subscriber(peripherals: pins::LedPeripherals) {
    use riot_rs::sensors::interrupts::{
        AccelerometerInterruptEvent, DeviceInterrupt, InterruptEvent, InterruptEventKind,
    };

    let event = InterruptEvent {
        kind: InterruptEventKind::Accelerometer(AccelerometerInterruptEvent::Movement),
        duration: None,
    };

    // // FIXME: set the pull resistor
    // // TODO: codegen this, or make this part of the sensor init
    // let interrupt_pin = gpio::Input::new(peripherals.i2c_int_int, gpio::Pull::None);

    // // FIXME: change Int1
    // sensors::ACCEL
    //     .register_interrupt_pin(interrupt_pin, DeviceInterrupt::Int1)
    //     .await
    //     .unwrap();

    // let accel = REGISTRY
    //     .sensors()
    //     .find(|s| s.categories().contains(&riot_rs::sensors::Category::Accelerometer))
    //     .unwrap();

    use riot_rs::sensors::Sensor;

    let mut led1 = Output::new(peripherals.led_row1, Level::Low);

    // The micro:bit uses an LED matrix; pull the column line low.
    #[cfg(context = "microbit-v2")]
    let _led_col1 = Output::new(peripherals.led_col1, Level::Low);

    loop {
        led1.set_low();
        println!("Waiting for free fall");
        sensors::ACCEL
            .wait_for_interrupt_event(event)
            .await
            .unwrap();
        led1.set_high();
        println!("Interrupt!");

        if sensors::ACCEL.trigger_measurement().is_ok() {
            let reading = sensors::ACCEL.wait_for_reading().await.unwrap();
            println!("reading: {:?}", reading);
        }

        // Timer::after(Duration::from_millis(500)).await;

        // FIXME: turn an LED on
        // FIXME: turn the LED off when we're not free falling anymore
        // FIXME: use the magnetometer in one-shot mode to get the orientation
    }
}
