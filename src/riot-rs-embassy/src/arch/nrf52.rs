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
    use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
    use riot_rs_saga::sensor::{PhysicalUnit, PhysicalValue, Reading, ReadingResult, Sensor};

    embassy_nrf::bind_interrupts!(struct Irqs {
        TEMP => embassy_nrf::temp::InterruptHandler;
    });

    pub struct InternalTemp {
        temp: Mutex<CriticalSectionRawMutex, Option<temp::Temp<'static>>>,
    }

    impl InternalTemp {
        pub const fn new() -> Self {
            Self {
                temp: Mutex::new(None),
            }
        }

        pub fn init(&self, peripheral: peripherals::TEMP) {
            // FIXME: we use try_lock instead of lock to not make this function async, can we do
            // better?
            // FIXME: return an error when relevant
            let mut temp = self.temp.try_lock().unwrap();
            *temp = Some(temp::Temp::new(peripheral, Irqs));
        }
    }

    // pub struct TemperatureReading(PhysicalValue);
    //
    // impl Reading for TemperatureReading {
    //     fn value(&self) -> PhysicalValue {
    //         self.0
    //     }
    // }

    impl Sensor for InternalTemp {
        fn read(&self) -> ReadingResult<PhysicalValue> {
            use fixed::traits::LossyInto;

            let reading = embassy_futures::block_on(async {
                self.temp.lock().await.as_mut().unwrap().read().await
            });

            let temp: i32 = (100 * reading).lossy_into();

            Ok(PhysicalValue {
                value: temp,
            })
        }

        fn value_scale() -> i8 {
            -2
        }

        fn unit() -> PhysicalUnit {
            PhysicalUnit::Celsius
        }

        fn display_name() -> Option<&'static str> {
            Some("Internal temperature sensor")
        }

        fn part_number() -> &'static str {
            "nrf52 internal temperature sensor"
        }

        fn version() -> u8 {
            0
        }
    }
}
