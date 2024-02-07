use picoserve::{
    extract::State,
    response::{IntoResponse, Json},
};

use crate::ButtonInputs;

pub async fn buttons(State(ButtonInputs(button_inputs)): State<ButtonInputs>) -> impl IntoResponse {
    let buttons = button_inputs.lock().await;

    Json(JsonButtons {
        button1: buttons.0.get(0).unwrap().is_low(),
        button2: buttons.0.get(1).unwrap().is_low(),
        button3: buttons.0.get(2).unwrap().is_low(),
        button4: buttons.0.get(3).unwrap().is_low(),
    })
}

#[derive(serde::Serialize)]
struct JsonButtons {
    button1: bool,
    button2: bool,
    button3: bool,
    button4: bool,
}
