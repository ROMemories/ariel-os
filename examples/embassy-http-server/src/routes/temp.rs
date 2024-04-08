use picoserve::{
    extract::State,
    response::{IntoResponse, Json},
};
use riot_rs::sensors::sensor::{Reading, Sensor};

use crate::TEMP_SENSOR;

pub async fn temp() -> impl IntoResponse {
    // FIXME: handle this unwrap
    let temp = TEMP_SENSOR.read().await.unwrap().value;

    Json(JsonTemp { temp })
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
}
