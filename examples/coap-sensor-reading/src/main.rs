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

    let handler = coap_handler_implementations::new_dispatcher()
        .at_with_attributes(
            &["sensors", "temp"],
            &[
                Attribute::Title("Temperature Sensor"), // From RFC5988 + RFC6690.
            ],
            TypeHandler::new_minicbor_2(coap_handler_implementations::with_get(SensorInfoRenderer)),
        )
        .at_with_attributes(
            &["sensors", "temp", "reading"],
            &[
                Attribute::Title("Temperature Sensor Reading"), // From RFC5988 + RFC6690.
                Attribute::Interface("sensor"),                 // From RFC6690.
                Attribute::ResourceType("temperature-c"),       // From RFC6690.
            ],
            TypeHandler::new_minicbor_2(coap_handler_implementations::with_get(
                SensorReadingRenderer,
            )),
        );

    ariel_os::coap::coap_run(handler).await;
}

struct SensorReadingRenderer;

impl GetRenderable for SensorReadingRenderer {
    type Get = f32; // TODO

    fn get(&mut self) -> Result<Self::Get, coap_message_utils::Error> {
        let Some((reading_channel, sample)) = SENSOR_READING.try_take() else {
            // FIXME: likely not the right response when no reading ready yet.
            return Err(coap_message_utils::Error::service_unavailable());
        };

        let Ok(value) = sample.value() else {
            // FIXME: likely not the right response.
            return Err(coap_message_utils::Error::service_unavailable());
        };

        let channel_scaling = i32::from(reading_channel.scaling());
        let factor = 10i32.pow(channel_scaling.unsigned_abs()) as f32;
        let value = if channel_scaling < 0 {
            value as f32 / factor
        } else {
            value as f32 * factor
        };

        Ok(value)
    }
}

#[ariel_os::task(autostart)]
async fn sensor_loop() {
    loop {
        let Some(sensor) = get_temp_sensor() else {
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

struct SensorInfoRenderer;

impl GetRenderable for SensorInfoRenderer {
    type Get = SensorInfo;

    fn get(&mut self) -> Result<Self::Get, coap_message_utils::Error> {
        let Some(sensor) = get_temp_sensor() else {
            return Err(coap_message_utils::Error::service_unavailable());
        };

        Ok(SensorInfo {
            label: sensor.label(),
            display_name: sensor.display_name(),
            part_number: sensor.part_number(),
            version: sensor.version(),
        })
    }
}

#[derive(Debug, Clone)]
struct SensorInfo {
    label: Option<&'static str>, // TODO: consider using this as an `rt` CoRE Link Attribute as well.
    display_name: Option<&'static str>,
    part_number: Option<&'static str>,
    version: u8,
}

impl<C> minicbor::Encode<C> for SensorInfo {
    fn encode<W: minicbor::encode::Write>(
        &self,
        e: &mut minicbor::encode::Encoder<W>,
        _ctx: &mut C,
    ) -> Result<(), minicbor::encode::Error<W::Error>> {
        // Number of fields to encode.
        e.map(4)?;

        e.str("lb")?;

        if let Some(label) = self.label {
            e.str(label)?;
        } else {
            e.null()?;
        }

        e.str("dn")?;

        if let Some(display_name) = self.display_name {
            e.str(display_name)?;
        } else {
            e.null()?;
        }

        e.str("pn")?;

        if let Some(part_number) = self.part_number {
            e.str(part_number)?;
        } else {
            e.null()?;
        }

        e.str("ver")?;

        e.u8(self.version)?;

        Ok(())
    }
}

fn get_temp_sensor() -> Option<&'static dyn ariel_os::sensors::Sensor> {
    ariel_os::sensors::REGISTRY.sensors().find(|s| {
        s.categories()
            .iter()
            .any(|c| [Category::Temperature, Category::RelativeHumidityTemperature].contains(c))
    })
}
