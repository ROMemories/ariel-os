//! This entire module is intended to be entirely auto-generated.

pub static LSM6DSV16X_I2C: ariel_os_sensor_lsm6dsv16x::Lsm6dsv16xI2c =
    const { ariel_os_sensor_lsm6dsv16x::Lsm6dsv16xI2c::new(Some("onboard")) };
#[ariel_os::reexports::linkme::distributed_slice(ariel_os::sensors::SENSOR_REFS)]
static LSM6DSV16X_I2C_REF: &'static dyn ariel_os::sensors::Sensor = &LSM6DSV16X_I2C;

#[ariel_os::task]
pub async fn lsm6dsv16x_i2c_runner() {
    LSM6DSV16X_I2C.run().await
}

pub static STTS22H_I2C: ariel_os_sensor_stts22h::Stts22hI2c =
    const { ariel_os_sensor_stts22h::Stts22hI2c::new(Some("onboard")) };
#[ariel_os::reexports::linkme::distributed_slice(ariel_os::sensors::SENSOR_REFS)]
static STTS22H_I2C_REF: &'static dyn ariel_os::sensors::Sensor = &STTS22H_I2C;

#[ariel_os::task]
pub async fn stts22h_i2c_runner() {
    STTS22H_I2C.run().await
}
