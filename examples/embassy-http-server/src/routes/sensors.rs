use picoserve::response::IntoResponse;
use riot_rs::sensors::{Sensor, REGISTRY};

pub async fn sensors() -> impl IntoResponse {
    for sensor in REGISTRY.sensors() {
        if let (Ok(value), value_scale, unit, display_name) =
            riot_rs::await_read_sensor_main_value!(sensor)
        {
            let value = value.value() as f32 / 10i32.pow((-value_scale) as u32) as f32;
            riot_rs::debug::println!("{}: {} {}", display_name.unwrap(), value, unit);
        } else {
            return "Error reading sensor";
        }
    }

    "No sensors"
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
}
