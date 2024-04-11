use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use picoserve::{
    extract::State,
    response::{IntoResponse, Json},
};
use riot_rs::{
    sensors::sensor::{Reading, Sensor},
    thread,
};

use crate::{println, TEMP_SENSOR};

pub async fn temp() -> impl IntoResponse {
    let signal: Signal<CriticalSectionRawMutex, i32> = Signal::new();

    fn read_temp(signal: &Signal<CriticalSectionRawMutex, i32>) {
        // FIXME: handle this unwrap
        let temp = TEMP_SENSOR.read().unwrap().value;
        // println!("read");
        signal.signal(temp);
        // println!("signaled");
    }

    let mut stack = [0u8; 2048_usize];
    // println!("will spawn");
    thread::thread_create(read_temp, &signal, &mut stack, 1);
    // println!("thread spawned");

    let temp = signal.wait().await;
    // println!("waited");

    Json(JsonTemp { temp })
}

#[thread]
fn _dummy() {
    thread::sleep();
}

#[derive(serde::Serialize)]
struct JsonTemp {
    // Temperature in hundredths of degrees Celsius
    temp: i32,
}
