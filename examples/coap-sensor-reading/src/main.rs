#![no_main]
#![no_std]

mod i2c_bus;
mod pins;
mod sensors;

use ariel_os::{
    log::{error, info, warn},
    sensors::{
        Category, Label, MeasurementUnit, Reading as _,
        sensor::{ReadingChannel, Sample},
    },
    time::Timer,
};
use coap_handler::Attribute;
use coap_handler_implementations::{GetRenderable, TypeHandler};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

static SENSOR_READING: Signal<CriticalSectionRawMutex, (ReadingChannel, Sample)> = Signal::new();

#[ariel_os::task(autostart, peripherals)]
async fn main(peripherals: pins::Peripherals) {
    use coap_handler_implementations::HandlerBuilder as _;

    i2c_bus::init(peripherals);
    sensors::init().await;

    let handler = coap_handler_implementations::new_dispatcher().at_with_attributes(
        &["sensors", "temp", "reading"],
        &[
            Attribute::Title("Temperature Sensor Reading"), // From RFC5988 + RFC6690.
            Attribute::Interface("sensor"),                 // From RFC6690.
            Attribute::ResourceType("temperature-c"),       // From RFC6690.
        ],
        TypeHandler::new_minicbor_2(coap_handler_implementations::with_get(
            SensorReadingRenderer::new(),
        )),
    );

    ariel_os::coap::coap_run(handler).await;
}

struct SensorReadingRenderer {}

impl SensorReadingRenderer {
    fn new() -> Self {
        Self {}
    }
}

impl GetRenderable for SensorReadingRenderer {
    type Get = i32; // TODO

    fn get(&mut self) -> Result<Self::Get, coap_message_utils::Error> {
        let Some((reading_channel, sample)) = SENSOR_READING.try_take() else {
            // FIXME: likely not the right response when no reading ready yet.
            return Err(coap_message_utils::Error::service_unavailable());
        };

        let Ok(value) = sample.value() else {
            // FIXME: likely not the right response.
            return Err(coap_message_utils::Error::service_unavailable());
        };

        Ok(value)
    }
}

#[ariel_os::task(autostart)]
async fn sensor_loop() {
    loop {
        let Some(sensor) = ariel_os::sensors::REGISTRY.sensors().find(|s| {
            s.categories()
                .iter()
                .any(|c| [Category::Temperature, Category::RelativeHumidityTemperature].contains(c))
        }) else {
            info!("There aren't any registered temperature sensors");
            break;
        };

        if let Err(err) = sensor.trigger_measurement() {
            warn!("Error when triggering a measurement: {}", err);
            Timer::after_secs(2).await;
            continue;
        }
        let reading = sensor.wait_for_reading().await;

        match reading {
            Ok(samples) => {
                for (reading_channel, sample) in samples
                    .samples()
                    .filter(|(reading_channel, _)| reading_channel.label() == Label::Temperature)
                {
                    // Our code only supports Celsius right now
                    match reading_channel.unit() {
                        MeasurementUnit::Celsius => {
                            SENSOR_READING.signal((reading_channel, sample));
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => {
                error!("Error when reading: {}", err);
            }
        }
        Timer::after_secs(2).await; // TODO: increase this.
    }
}
