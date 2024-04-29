use embassy_executor::Spawner;
use riot_rs::embassy::{arch, arch::peripherals};

// TODO: this whole module should get auto-generated

#[cfg(context = "nrf52")]
pub static TEMP_SENSOR: riot_rs::embassy::arch::internal_temp::InternalTemp =
    riot_rs::embassy::arch::internal_temp::InternalTemp::new();
#[cfg(context = "nrf52")]
#[riot_rs::linkme::distributed_slice(riot_rs::sensors::SENSOR_REFS)]
#[linkme(crate = riot_rs::linkme)]
static TEMP_SENSOR_REF: &'static dyn riot_rs::sensors::sensor::Sensor = &TEMP_SENSOR;

#[cfg(context = "nrf52")]
#[riot_rs::spawner(autostart, peripherals)]
fn temp_sensor_init(spawner: Spawner, peripherals: TempPeripherals) {
    TEMP_SENSOR.init(spawner, peripherals.p);
}

#[cfg(context = "nrf52")]
riot_rs::define_peripherals!(TempPeripherals { p: TEMP });

#[cfg(feature = "button-readings")]
pub static BUTTON_1: riot_rs::builtin_sensors::push_buttons::PushButton =
    riot_rs::builtin_sensors::push_buttons::PushButton::new();
#[cfg(feature = "button-readings")]
#[riot_rs::linkme::distributed_slice(riot_rs::sensors::SENSOR_REFS)]
#[linkme(crate = riot_rs::linkme)]
static BUTTON_1_REF: &'static dyn riot_rs::sensors::sensor::Sensor = &BUTTON_1;

#[cfg(feature = "button-readings")]
#[riot_rs::spawner(autostart, peripherals)]
fn button_1_init(_spawner: Spawner, peripherals: Button1Peripherals) {
    // FIXME: how to codegen this?
    BUTTON_1.init(riot_rs::embassy::arch::gpio::Input::new(
        peripherals.p,
        riot_rs::embassy::arch::gpio::Pull::Up,
    ));
}

#[cfg(feature = "button-readings")]
riot_rs::define_peripherals!(Button1Peripherals { p: P0_11 });

#[cfg(context = "nrf52840")]
pub static ACCEL: riot_rs::builtin_sensors::lis3dh::Lis3dh<arch::i2c::I2c> =
    riot_rs::builtin_sensors::lis3dh::Lis3dh::<arch::i2c::I2c>::new();
#[cfg(context = "nrf52840")]
#[riot_rs::linkme::distributed_slice(riot_rs::sensors::SENSOR_REFS)]
#[linkme(crate = riot_rs::linkme)]
static ACCEL_REF: &'static dyn riot_rs::sensors::sensor::Sensor = &ACCEL;

#[cfg(context = "nrf52840")]
#[riot_rs::spawner(autostart, peripherals)]
fn accel_init(spawner: Spawner, peripherals: AccelPeripherals) {
    let i2c_dev = arch::i2c::I2cDevice::new(riot_rs::embassy::I2C_BUS.get().unwrap());
    ACCEL.init(spawner, i2c_dev);
}

#[cfg(context = "nrf52840")]
riot_rs::define_peripherals!(AccelPeripherals {
    // i2c: TWISPI0,
    // sda: P0_30,
    // scl: P0_31
});
