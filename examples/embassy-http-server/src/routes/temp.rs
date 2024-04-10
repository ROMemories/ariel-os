use picoserve::{
    extract::State,
    response::{IntoResponse, Json},
};
use riot_rs::sensors::sensor::{Reading, Sensor};

use crate::TEMP_SENSOR;

pub async fn temp() -> impl IntoResponse {
    // FIXME: handle this unwrap
    // FIXME: this call to read() is blocking
    let temp = TEMP_SENSOR.read().unwrap().value;

    Json(JsonTemp { temp })
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
}
