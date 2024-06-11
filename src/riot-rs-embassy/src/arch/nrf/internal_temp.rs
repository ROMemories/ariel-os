// FIXME: use portable_atomic's instead
use core::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex};
use embassy_time::{Duration, Timer};
use riot_rs_sensors::{
    sensor::{
        Labels, MeasurementError, PhysicalUnits, PhysicalValue, PhysicalValues, ReadingError,
        ReadingResult, Sensor, ValueScales,
    },
    Category, Label, PhysicalUnit, Reading,
};

use crate::arch::peripherals;

embassy_nrf::bind_interrupts!(struct Irqs {
    TEMP => embassy_nrf::temp::InterruptHandler;
});

#[derive(Debug)]
#[non_exhaustive]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

crate::define_peripherals!(Peripherals { temp: TEMP });

pub struct InternalTemp {
    initialized: AtomicBool, // TODO: use an atomic bitset for initialized and enabled
    enabled: AtomicBool,
    label: Option<&'static str>,
    // TODO: use a blocking mutex instead?
    temp: Mutex<CriticalSectionRawMutex, Option<embassy_nrf::temp::Temp<'static>>>,
    channel: Channel<CriticalSectionRawMutex, Notification, 1>,
    // feature is not used
    lower_threshold: AtomicI32,
    lower_threshold_enabled: AtomicBool, // TODO: use an atomic bitset for handler other
                                         // thresholds
}

impl InternalTemp {
    #[must_use]
    pub const fn new(label: Option<&'static str>) -> Self {
        Self {
            initialized: AtomicBool::new(false),
            enabled: AtomicBool::new(false),
            label,
            temp: Mutex::new(None),
            channel: Channel::new(),
            lower_threshold: AtomicI32::new(0),
            lower_threshold_enabled: AtomicBool::new(false),
        }
    }

    pub fn init(&'static self, spawner: Spawner, peripherals: Peripherals, config: Config) {
        if !self.initialized.load(Ordering::Acquire) {
            // We use `try_lock()` instead of `lock()` to not make this function async.
            // This mutex cannot be locked at this point as it is private and can only be
            // locked when the sensor has been initialized successfully.
            let mut temp = self.temp.try_lock().unwrap();
            *temp = Some(embassy_nrf::temp::Temp::new(peripherals.temp, Irqs));

            #[embassy_executor::task]
            async fn temp_watcher(sensor: &'static InternalTemp) {
                loop {
                    if sensor.lower_threshold_enabled.load(Ordering::Acquire) {
                        if let Ok(value) = sensor.read().await {
                            if value.value().value()
                                > sensor.lower_threshold.load(Ordering::Acquire)
                            {
                                // FIXME: should this be Lower or Higher?
                                let _ = sensor
                                    .channel
                                    .try_send(Notification::Threshold(ThresholdKind::Lower));
                                riot_rs_debug::println!("Temp > lower threshold: {:?}", value);
                            }
                        }
                    }
                    // TODO: make this duration configurable?
                    // Avoid busy looping and allow other users to lock the mutex
                    Timer::after(Duration::from_millis(100)).await;
                }
            }
            spawner.spawn(temp_watcher(&self)).unwrap();

            self.initialized.store(true, Ordering::Release);
            self.enabled.store(true, Ordering::Release);
        }
    }
}

impl Sensor for InternalTemp {
    async fn read(&self) -> ReadingResult<PhysicalValues> {
        const ERROR: MeasurementError = MeasurementError::Symmetrical {
            deviation: 5,
            bias: 0,
            scale: 0,
        };

        //     self.read().await.map(|v| v.value())
        // }
        //
        // #[allow(refining_impl_trait)]
        // async fn read(&self) -> ReadingResult<TemperatureReading> {
        use fixed::traits::LossyInto;

        if !self.enabled.load(Ordering::Acquire) {
            return Err(ReadingError::Disabled);
        }

        let reading = self.temp.lock().await.as_mut().unwrap().read().await;
        let temp: i32 = (100 * reading).lossy_into();

        Ok(PhysicalValues::V1([PhysicalValue::new(temp, ERROR)]))
    }

    fn set_enabled(&self, enabled: bool) {
        if self.initialized.load(Ordering::Acquire) {
            self.enabled.store(enabled, Ordering::Release);
        }
        // TODO: return an error otherwise?
    }

    fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Acquire)
    }

    fn set_threshold(&self, kind: ThresholdKind, value: PhysicalValue) {
        match kind {
            ThresholdKind::Lower => self.lower_threshold.store(value.value(), Ordering::Release),
            _ => {
                // TODO: should we return an error instead?
            }
        }
    }

    fn set_threshold_enabled(&self, kind: ThresholdKind, enabled: bool) {
        match kind {
            ThresholdKind::Lower => self
                .lower_threshold_enabled
                .store(enabled, Ordering::Release),
            _ => {
                // TODO: should we return an error instead?
            }
        }
    }

    fn subscribe(&self) -> NotificationReceiver {
        // TODO: receiver competes for notification: limit the number of receivers to 1?
        self.channel.receiver()
    }

    fn categories(&self) -> &'static [Category] {
        &[Category::Temperature]
    }

    fn value_scales(&self) -> ValueScales {
        ValueScales::V1([-2])
    }

    fn reading_labels(&self) -> Labels {
        Labels::V1([Label::Main])
    }

    fn label(&self) -> Option<&'static str> {
        self.label
    }

    fn units(&self) -> PhysicalUnits {
        PhysicalUnits::V1([PhysicalUnit::Celsius])
    }

    fn display_name(&self) -> Option<&'static str> {
        Some("Internal temperature sensor")
    }

    fn part_number(&self) -> &'static str {
        "nrf52 internal temperature sensor"
    }

    fn version(&self) -> u8 {
        0
    }
}
