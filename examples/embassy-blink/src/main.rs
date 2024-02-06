#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

mod pins;

use embassy_time::{Duration, Timer};
use riot_rs::embassy::{arch, Application, ApplicationError, Drivers};

use crate::leds::Leds;

#[embassy_executor::task]
async fn blink(mut leds: leds::Leds) {
    loop {
        for led in leds.get_mut() {
            led.toggle();
        }
        Timer::after(Duration::from_millis(500)).await;
    }
}

struct Blink;

impl Application for Blink {
    fn init() -> &'static dyn Application {
        &Self {}
    }

    fn start(
        &self,
        peripherals: &mut arch::OptionalPeripherals,
        spawner: embassy_executor::Spawner,
        _drivers: Drivers,
    ) -> Result<(), ApplicationError> {
        let leds = pins::Leds::take_from(peripherals)?;

        let leds = Leds::new(leds);

        spawner.spawn(blink(leds)).unwrap();

        Ok(())
    }
}

riot_rs::embassy::riot_initialize!(Blink);

mod leds {
    use embassy_nrf::gpio::{AnyPin, Level, Output, OutputDrive, Pin};

    use crate::pins;

    pub const LED_COUNT: usize = 4;

    pub struct Led(Output<'static>);

    impl Led {
        pub fn new(led: AnyPin) -> Self {
            Self(Output::new(led, Level::Low, OutputDrive::Standard))
        }

        pub fn toggle(&mut self) {
            self.0.toggle()
        }
    }

    pub struct Leds([Led; LED_COUNT]);

    impl Leds {
        pub fn new(led_peripherals: pins::Leds) -> Self {
            Self([
                Led::new(led_peripherals.led1.degrade()),
                Led::new(led_peripherals.led2.degrade()),
                Led::new(led_peripherals.led3.degrade()),
                Led::new(led_peripherals.led4.degrade()),
            ])
        }

        pub fn get_mut(&mut self) -> &mut [Led] {
            &mut self.0
        }
    }
}
