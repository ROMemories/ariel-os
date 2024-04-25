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

riot_rs::define_peripherals!(Button1Peripherals { p: P0_11 });

// FIXME: move the Twim instantiation into arch
pub static ACCEL: riot_rs::builtin_sensors::lis3dh::Lis3dh =
    riot_rs::builtin_sensors::lis3dh::Lis3dh::new();
#[riot_rs::linkme::distributed_slice(riot_rs::sensors::SENSOR_REFS)]
#[linkme(crate = riot_rs::linkme)]
static ACCEL_REF: &'static dyn riot_rs::sensors::sensor::Sensor = &ACCEL;

#[riot_rs::spawner(autostart, peripherals)]
fn accel_init(spawner: Spawner, peripherals: AccelPeripherals) {
    // FIXME: how to codegen this?
    // FIXME: get the config from hw-setup.yml
    let mut config = arch::i2c::Config::default();
    config.frequency = arch::i2c::Frequency::K100;
    config.scl_high_drive = false;
    config.sda_pullup = false;
    config.sda_high_drive = false;
    config.scl_pullup = false;

    let i2c =
        arch::i2c::I2c::new(peripherals.i2c, peripherals.sda, peripherals.scl, config);
    ACCEL.init(spawner, i2c);
}

riot_rs::define_peripherals!(AccelPeripherals { i2c: TWISPI0, sda: P0_30, scl: P0_31});
