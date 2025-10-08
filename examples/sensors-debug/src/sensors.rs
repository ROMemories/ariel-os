//! This entire module is intended to be entirely auto-generated.

pub static LIS2DU12_I2C: ariel_os_sensor_lis2du12::Lis2du12I2c =
    const { ariel_os_sensor_lis2du12::Lis2du12I2c::new(Some("onboard")) };
#[ariel_os::reexports::linkme::distributed_slice(ariel_os::sensors::SENSOR_REFS)]
static LIS2DU12_I2C_REF: &'static dyn ariel_os::sensors::Sensor = &LIS2DU12_I2C;

#[ariel_os::task]
pub async fn lis2du12_i2c_runner() {
    LIS2DU12_I2C.run().await
}

pub static LIS2MDL_I2C: ariel_os_sensor_lis2mdl::Lis2mdlI2c =
    const { ariel_os_sensor_lis2mdl::Lis2mdlI2c::new(Some("onboard")) };
#[ariel_os::reexports::linkme::distributed_slice(ariel_os::sensors::SENSOR_REFS)]
static LIS2MDL_I2C_REF: &'static dyn ariel_os::sensors::Sensor = &LIS2MDL_I2C;

#[ariel_os::task]
pub async fn lis2mdl_i2c_runner() {
    LIS2MDL_I2C.run().await
}

pub static LPS22DF_I2C: ariel_os_sensor_lps22df::Lps22dfI2c =
    const { ariel_os_sensor_lps22df::Lps22dfI2c::new(Some("onboard")) };
#[ariel_os::reexports::linkme::distributed_slice(ariel_os::sensors::SENSOR_REFS)]
static LPS22DF_I2C_REF: &'static dyn ariel_os::sensors::Sensor = &LPS22DF_I2C;

#[ariel_os::task]
pub async fn lps22df_i2c_runner() {
    LPS22DF_I2C.run().await
}

pub static LSM6DSV16X_I2C: ariel_os_sensor_lsm6dsv16x::Lsm6dsv16xI2c =
    const { ariel_os_sensor_lsm6dsv16x::Lsm6dsv16xI2c::new(Some("onboard")) };
#[ariel_os::reexports::linkme::distributed_slice(ariel_os::sensors::SENSOR_REFS)]
static LSM6DSV16X_I2C_REF: &'static dyn ariel_os::sensors::Sensor = &LSM6DSV16X_I2C;

#[ariel_os::task]
pub async fn lsm6dsv16x_i2c_runner() {
    LSM6DSV16X_I2C.run().await
}

pub static STTS22H_I2C: ariel_os_sensor_stts22h::Stts22hI2c =
    const { ariel_os_sensor_stts22h::Stts22hI2c::new(Some("indoor")) };
#[ariel_os::reexports::linkme::distributed_slice(ariel_os::sensors::SENSOR_REFS)]
static STTS22H_I2C_REF: &'static dyn ariel_os::sensors::Sensor = &STTS22H_I2C;

#[ariel_os::task]
pub async fn stts22h_i2c_runner() {
    STTS22H_I2C.run().await
}
