/// Categories a sensor can be part of.
///
/// A sensor can be part of multiple categories.
// Built upon https://doc.riot-os.org/group__drivers__saul.html#ga8f2dfec7e99562dbe5d785467bb71bbb
// FIXME: rename this to class?
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize)]
pub enum Category {
    Accelerometer,
    Humidity,
    HumidityTemperature,
    PushButton,
    Temperature,
}
