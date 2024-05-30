use picoserve::response::{IntoResponse, Json};
use riot_rs::sensors::{PhysicalUnit, Reading, Sensor};

pub async fn buttons() -> impl IntoResponse {
    let reading = crate::sensors::BUTTON_1.read().await.unwrap().value();

    let is_pressed = match crate::sensors::BUTTON_1.units().first() {
        PhysicalUnit::ActiveOne => reading.value() == 1,
        PhysicalUnit::ActiveZero => reading.value() == 0,
        _ => unreachable!(),
    };

    Json(JsonButtons {
        button1: is_pressed,
    })
    // Json(JsonButtons {
    //     button1: buttons.button1.is_low(),
    //     button2: buttons.button2.is_low(),
    //     button3: buttons.button3.is_low(),
    //     button4: buttons.button4.is_low(),
    // })
}

#[derive(serde::Serialize)]
struct JsonButtons {
    button1: bool,
    // button2: bool,
    // button3: bool,
    // button4: bool,
}
