use picoserve::response::IntoResponse;
use riot_rs::sensors::{Reading, Sensor, REGISTRY};

pub async fn sensors() -> impl IntoResponse {
    for sensor in REGISTRY.sensors() {
        if let (Ok(values), value_scales, units, display_name, labels) =
            riot_rs::await_read_sensor_value!(sensor)
        {
            for (i, value) in values.values().enumerate() {
                let value_scale = value_scales.iter().nth(i).unwrap();
                let unit = units.iter().nth(i).unwrap();
                let label = labels.iter().nth(i).unwrap();
                let value = value.value() as f32 / 10i32.pow((-value_scale) as u32) as f32;
                riot_rs::debug::println!("{}: {} {} ({})", display_name.unwrap(), value, unit, label);
            }
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
