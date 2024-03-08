use picoserve::{
    extract::State,
    response::{IntoResponse, Json},
};
use riot_rs::saga::Sensor;

use crate::TempInput;

pub async fn temp(State(TempInput(temp)): State<TempInput>) -> impl IntoResponse {
    // FIXME: avoid having to use unwrap to extract values
    let temp = temp.lock().await.read().await.0.get(0).unwrap().value;

    Json(JsonTemp { temp })
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i16,
}
