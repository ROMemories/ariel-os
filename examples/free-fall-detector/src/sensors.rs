//! This whole module should eventually be codegened.

use riot_rs::arch::peripherals;

pub use sensors::*;

/// Type alias of this sensor instance
pub type Lsm303agr_ACCEL = riot_rs_builtin_sensors::lsm303agr::Lsm303agrI2c;

// Instantiate the sensor driver
pub static ACCEL: Lsm303agr_ACCEL = Lsm303agr_ACCEL::new(Some("accel"));

// Store a static reference in the sensor distributed slice
#[riot_rs::reexports::linkme::distributed_slice(riot_rs::sensors::SENSOR_REFS)]
#[linkme(crate = riot_rs::reexports::linkme)]
static ACCEL_REF: &'static dyn riot_rs::sensors::Sensor = &ACCEL;

// #[cfg(context = "microbit-v2")]
mod sensors {
    use embassy_time::{Duration, Timer};

    use super::ACCEL;

    #[riot_rs::task(autostart, peripherals)]
    async fn ACCEL_run(peripherals: crate::pins::ACCEL_Peripherals) {
        use embedded_hal_async::i2c::I2c;

        let mut config = riot_rs_builtin_sensors::lsm303agr::Config::default();

        let mut i2c_device =
            riot_rs::i2c::controller::I2cDevice::new(crate::buses::I2C0.get().unwrap());

        let spawner = riot_rs::Spawner::for_current_executor().await;
        ACCEL.init(spawner, peripherals.p, i2c_device, config).await;

        let mut i2c_device =
            riot_rs::i2c::controller::I2cDevice::new(crate::buses::I2C0.get().unwrap());
        const TARGET_I2C_ADDR: u8 = 0x19;

        const CTRL_REG1_A: u8 = 0x20;
        i2c_device
            .write(TARGET_I2C_ADDR, &[CTRL_REG1_A, 0xa7])
            .await
            .unwrap();

        const CTRL_REG2_A: u8 = 0x21;
        i2c_device
            .write(TARGET_I2C_ADDR, &[CTRL_REG2_A, 0x00])
            .await
            .unwrap();

        // Enable INT1
        const CTRL_REG3_A: u8 = 0x22;
        i2c_device
            .write(TARGET_I2C_ADDR, &[CTRL_REG3_A, 0x40])
            .await
            .unwrap();

        // Set FS
        const CTRL_REG4_A: u8 = 0x23;
        i2c_device
            .write(TARGET_I2C_ADDR, &[CTRL_REG4_A, 0x00])
            .await
            .unwrap();

        // Set FS
        const CTRL_REG5_A: u8 = 0x24;
        i2c_device
            .write(TARGET_I2C_ADDR, &[CTRL_REG5_A, 0x00])
            .await
            .unwrap();

        // //
        // const CTRL_REG6_A: u8 = 0x25;
        // i2c_device.write(TARGET_I2C_ADDR, &[CTRL_REG6_A, 0x02]).await.unwrap();

        // Set the free-fall threshold
        const INT1_THS_A: u8 = 0x32;
        i2c_device
            .write(TARGET_I2C_ADDR, &[INT1_THS_A, 0x16])
            .await
            .unwrap();

        // Set the free-fall event duration
        const INT1_DURATION_A: u8 = 0x33;
        i2c_device
            .write(TARGET_I2C_ADDR, &[INT1_DURATION_A, 0x03])
            .await
            .unwrap();

        // Configure free-fall recognition
        const INT1_CFG_A: u8 = 0x30;
        i2c_device
            .write(TARGET_I2C_ADDR, &[INT1_CFG_A, 0x95])
            .await
            .unwrap();

        let pull = riot_rs::gpio::Pull::Up;
        let mut int1 = riot_rs::gpio::Input::builder(peripherals.int.i2c_int_int, pull)
            .build_with_interrupt()
            .unwrap();

        const INTERFACE_MCU_ADDR: u8 = 0x70;

        let mut i2c_device =
            riot_rs::i2c::controller::I2cDevice::new(crate::buses::I2C0.get().unwrap());

        loop {
            Timer::after(Duration::from_millis(10)).await;

            let mut buf = [0; 4];
            // TODO: actually maybe we should wait for the interrupt signal before reading?
            i2c_device
                .write(INTERFACE_MCU_ADDR, &[0x10, 0x06])
                .await
                .unwrap();
            int1.wait_for_low().await;
            i2c_device.read(INTERFACE_MCU_ADDR, &mut buf).await.unwrap();
            crate:: println!("Interface MCU response: {:x?}", buf);
            // In case of non-error response
            if buf[0] == 0x11 {
                break;
            }
        }

        // FIXME: change Int1
        ACCEL.register_interrupt_pin(int1, riot_rs::sensors::interrupts::DeviceInterrupt::Int1);
        // ACCEL.use_interrupt_for_data_ready_event();

        ACCEL.run().await
    }
}
