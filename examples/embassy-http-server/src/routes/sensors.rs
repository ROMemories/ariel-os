use picoserve::response::IntoResponse;
use riot_rs::sensors::{Reading, REGISTRY};

pub async fn sensors() -> impl IntoResponse {
    for sensor in REGISTRY.sensors() {
        match riot_rs::sensors::measure!(sensor).await {
            Ok(values) => {
                for (i, value) in values.values().enumerate() {
                    let value_scale = sensor.value_scales().iter().nth(i).unwrap();
                    let unit = sensor.units().iter().nth(i).unwrap();
                    let reading_label = sensor.reading_labels().iter().nth(i).unwrap();
                    let value = value.value() as f32 / 10i32.pow((-value_scale) as u32) as f32;
                    riot_rs::debug::println!(
                        "{} ({}): {} {} ({})",
                        sensor.display_name().unwrap_or("unknown"),
                        sensor.label().unwrap_or("no label"),
                        value,
                        unit,
                        reading_label
                    );
                }
            }
            Err(err) => {
                riot_rs::debug::println!("error while reading sensor value: {}", err);
                return "Error reading sensor";
            }
        }
    }

    "No sensors"
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
}
