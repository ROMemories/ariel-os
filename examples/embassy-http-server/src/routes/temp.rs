use picoserve::response::{IntoResponse, Json};
use riot_rs::sensors::{Category, PhysicalUnit, Reading, REGISTRY};

pub async fn temp() -> impl IntoResponse {
    let temp_sensor = REGISTRY
        .sensors()
        .find(|s| s.categories().contains(&Category::Temperature) && s.label() == Some("internal"))
        .unwrap();
    let reading = riot_rs::sensors::measure!(temp_sensor)
        .await
        .unwrap()
        .value();
    let temp =
        reading.value() as f32 / 10i32.pow((-temp_sensor.value_scales().first()) as u32) as f32;
    let unit = temp_sensor.units().first();

    Json(JsonTemp { temp, unit })
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: f32,
    unit: PhysicalUnit,
}
