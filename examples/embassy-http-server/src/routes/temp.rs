use picoserve::response::{IntoResponse, Json};
use riot_rs::sensors::{categories::temperature::TemperatureSensor, PhysicalUnit, Sensor};

use crate::sensors::TEMP_SENSOR;

pub async fn temp() -> impl IntoResponse {
    let temp = TEMP_SENSOR
        .read_temperature()
        .await
        .unwrap()
        .temperature()
        .value();
    let temp = temp as f32 / 10i32.pow((-TEMP_SENSOR.value_scale()) as u32) as f32;
    let unit = TEMP_SENSOR.unit();

    Json(JsonTemp { temp, unit })
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: f32,
    unit: PhysicalUnit,
}
