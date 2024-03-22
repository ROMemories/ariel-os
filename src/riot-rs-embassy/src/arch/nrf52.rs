pub(crate) use embassy_executor::InterruptExecutor as Executor;
pub use embassy_nrf::interrupt::SWI0_EGU0 as SWI;
pub use embassy_nrf::{config::Config, interrupt, peripherals, OptionalPeripherals};

#[interrupt]
unsafe fn SWI0_EGU0() {
    crate::EXECUTOR.on_interrupt()
}

#[cfg(feature = "usb")]
pub mod usb {
    use embassy_nrf::{
        bind_interrupts, peripherals, rng,
        usb::{
            self,
            vbus_detect::{self, HardwareVbusDetect},
            Driver,
        },
    };

    use crate::arch;

    bind_interrupts!(struct Irqs {
        USBD => usb::InterruptHandler<peripherals::USBD>;
        POWER_CLOCK => vbus_detect::InterruptHandler;
        RNG => rng::InterruptHandler<peripherals::RNG>;
    });

    pub type UsbDriver = Driver<'static, peripherals::USBD, HardwareVbusDetect>;

    pub fn driver(peripherals: &mut arch::OptionalPeripherals) -> UsbDriver {
        let usbd = peripherals.USBD.take().unwrap();
        Driver::new(usbd, Irqs, HardwareVbusDetect::new(Irqs))
    }
}

pub fn init(config: Config) -> OptionalPeripherals {
    let peripherals = embassy_nrf::init(config);
    OptionalPeripherals::from(peripherals)
}

#[cfg(feature = "internal-temp")]
pub mod internal_temp {
    use embassy_nrf::{peripherals, temp};
    use riot_rs_saga::{PhysicalUnit, PhysicalValue, Reading, ReadingResult, Sensor};

    embassy_nrf::bind_interrupts!(struct Irqs {
        TEMP => embassy_nrf::temp::InterruptHandler;
    });

    pub struct InternalTemp {
        temp: temp::Temp<'static>,
    }

    impl InternalTemp {
        pub fn new(peripheral: peripherals::TEMP) -> Self {
            let temp = temp::Temp::new(peripheral, Irqs);
            Self { temp }
        }
    }

    pub struct TemperatureReading(PhysicalValue);

    impl Reading for TemperatureReading {
        fn value(&self) -> PhysicalValue {
            self.0
        }
    }

    impl Sensor<TemperatureReading> for InternalTemp {
        async fn read(&mut self) -> ReadingResult<TemperatureReading> {
            use fixed::traits::LossyInto;

            let reading = self.temp.read().await;
            let temp: i32 = (100 * reading).lossy_into();

            Ok(TemperatureReading(PhysicalValue {
                value: i32::try_from(temp).unwrap(), // FIXME: remove this unwrap
            }))
        }

        fn value_scale() -> i8 {
            -2
        }

        fn unit() -> PhysicalUnit {
            PhysicalUnit::Celsius
        }
    }
}
