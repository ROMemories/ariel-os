//! This entire module is intended to be entirely auto-generated.

// pub static STTS22H_I2C: ariel_os_builtin_sensors::stts22h::Stts22hI2c =
//     const { ariel_os_builtin_sensors::stts22h::Stts22hI2c::new(Some("indoor")) };
// #[ariel_os::reexports::linkme::distributed_slice(ariel_os::sensors::SENSOR_REFS)]
// static STTS22H_I2C_REF: &'static dyn ariel_os::sensors::Sensor = &STTS22H_I2C;
//
// #[ariel_os::task]
// pub async fn stts22h_i2c_runner() {
//     STTS22H_I2C.run().await
// }

pub static HTS221_I2C: ariel_os_builtin_sensors::hts221::Hts221I2c =
    const { ariel_os_builtin_sensors::hts221::Hts221I2c::new(Some("indoor")) };
#[ariel_os::reexports::linkme::distributed_slice(ariel_os::sensors::SENSOR_REFS)]
static HTS221_I2C_REF: &'static dyn ariel_os::sensors::Sensor = &HTS221_I2C;

#[ariel_os::task]
pub async fn hts221_i2c_runner() {
    HTS221_I2C.run().await
}
