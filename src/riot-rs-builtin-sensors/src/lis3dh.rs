use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, mutex::Mutex, signal::Signal,
};
use embassy_time::{Duration, Timer};
use lis3dh_async::{Configuration, DataRate, Lis3dh as InnerLis3dh, Lis3dhI2C};
use portable_atomic::{AtomicU8, Ordering};
use riot_rs_embassy::{arch, gpio, i2c::controller::I2cDevice, Spawner};
use riot_rs_sensors::{
    interrupts::{
        self, AccelerometerInterruptEvent, DeviceInterrupt, InterruptError, InterruptEvent,
        InterruptEventKind,
    },
    sensor::{
        AccuracyError, MeasurementError, Mode as SensorMode, ModeSettingError, PhysicalValue,
        PhysicalValues, ReadingAxes, ReadingAxis, ReadingError, ReadingResult, ReadingWaiter,
        State, StateAtomic,
    },
    Category, Label, PhysicalUnit, Sensor,
};

// pub use crate::lis3dh_spi::Lis3dhSpi;
pub use lis3dh_async::{Mode, SlaveAddr as Address};

// FIXME: what's the best way to instantiate sensor driver configuration?
#[derive(Debug)]
#[non_exhaustive]
pub struct Config {
    pub address: Address,
    pub mode: Mode,
    pub datarate: DataRate,
    pub enable_x_axis: bool,
    pub enable_y_axis: bool,
    pub enable_z_axis: bool,
    pub block_data_update: bool, // TODO: do we need to expose this?
    pub enable_temperature: bool,
}

impl Default for Config {
    fn default() -> Self {
        let config = Configuration::default();
        Self {
            address: Address::Alternate,
            mode: config.mode,
            datarate: config.datarate,
            enable_x_axis: config.enable_x_axis,
            enable_y_axis: config.enable_y_axis,
            enable_z_axis: config.enable_z_axis,
            block_data_update: config.block_data_update,
            enable_temperature: config.enable_temperature,
        }
    }
}

riot_rs_embassy::define_peripherals!(Peripherals {});

const INTERRUPT_COUNT: usize = 2;

// TODO: could maybe use a OnceCell instead of an Option
pub struct Lis3dhI2c {
    state: StateAtomic,
    label: Option<&'static str>,
    data_rate: AtomicU8,
    // TODO: consider using MaybeUninit?
    accel: Mutex<CriticalSectionRawMutex, Option<InnerLis3dh<Lis3dhI2C<I2cDevice>>>>,
    trigger: Signal<CriticalSectionRawMutex, ()>,
    reading_channel: Channel<CriticalSectionRawMutex, ReadingResult<PhysicalValues>, 1>,
    interrupts: Mutex<CriticalSectionRawMutex, [Option<gpio::IntEnabledInput>; INTERRUPT_COUNT]>,
}

impl Lis3dhI2c {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new(label: Option<&'static str>) -> Self {
        // This is required to initialize the array because IntEnabledInput is not Copy.
        const NO_INTERRUPT: Option<gpio::IntEnabledInput> = None;
        Self {
            state: StateAtomic::new(State::Uninitialized),
            label,
            data_rate: AtomicU8::new(DataRate::PowerDown as u8),
            accel: Mutex::new(None),
            trigger: Signal::new(),
            reading_channel: Channel::new(),
            interrupts: Mutex::new([NO_INTERRUPT; INTERRUPT_COUNT]),
        }
    }

    pub async fn init(
        &'static self,
        _spawner: Spawner,
        peripherals: Peripherals,
        i2c: I2cDevice,
        config: Config,
    ) {
        if self.state.get() == State::Uninitialized {
            // TODO: can this be made shorter?
            let mut lis3dh_config = Configuration::default();
            lis3dh_config.mode = config.mode;
            lis3dh_config.datarate = config.datarate;
            lis3dh_config.enable_x_axis = config.enable_x_axis;
            lis3dh_config.enable_y_axis = config.enable_y_axis;
            lis3dh_config.enable_z_axis = config.enable_z_axis;
            lis3dh_config.block_data_update = config.block_data_update;
            lis3dh_config.enable_temperature = config.enable_temperature;

            // FIXME: is using block_on ok here?
            // FIXME: handle the Result
            // FIXME: this does not work because of https://github.com/embassy-rs/embassy/issues/2830
            // let init = embassy_futures::block_on(embassy_futures::select::select(
            //     InnerLis3dh::new_i2c_with_config(i2c, addr, config),
            //     Timer::after(Duration::from_secs(1)),
            // ));
            // let driver = match init {
            //     Either::First(driver) => driver.unwrap(),
            //     Either::Second(_) => panic!("timeout when initializing Lis3dh"), // FIXME
            // };

            let driver = InnerLis3dh::new_i2c_with_config(i2c, config.address, lis3dh_config)
                .await
                .unwrap();

            self.data_rate
                .store(lis3dh_config.datarate as u8, Ordering::Relaxed);

            // We use `try_lock()` instead of `lock()` to not make this function async.
            // This mutex cannot be locked at this point as it is private and can only be
            // locked when the sensor has been initialized successfully.
            let mut accel = self.accel.try_lock().unwrap();
            *accel = Some(driver);

            self.state.set(State::Enabled);
        }
    }

    pub async fn run(&self) -> ! {
        loop {
            let request = self.trigger.wait().await;

            // TODO: maybe should check is_data_ready()?
            let Ok(data) = self.accel.lock().await.as_mut().unwrap().accel_norm().await else {
                self.reading_channel
                    .send(Err(ReadingError::SensorAccess))
                    .await;
                continue;
            };

            #[expect(clippy::cast_possible_truncation)]
            // FIXME: dumb scaling, take precision into account
            // FIXME: specify the measurement error
            let x = PhysicalValue::new((data.x * 100.) as i32, AccuracyError::Unknown);
            let y = PhysicalValue::new((data.y * 100.) as i32, AccuracyError::Unknown);
            let z = PhysicalValue::new((data.z * 100.) as i32, AccuracyError::Unknown);

            self.reading_channel
                .send(Ok(PhysicalValues::V3([x, y, z])))
                .await;
        }
    }

    // FIXME: is this ok to make this method async?
    pub async fn register_interrupt_pin(
        &self,
        pin: gpio::IntEnabledInput,
        device_interrupt: DeviceInterrupt,
        event: InterruptEvent,
    ) -> Result<(), InterruptError> {
        // FIXME: check first that the interrupt can indeed handle that kind of event
        // (make a const function for that)

        let index = match device_interrupt {
            DeviceInterrupt::Int1 => 0,
            DeviceInterrupt::Int2 => 1,
            _ => {
                return Err(InterruptError::UnsupportedDeviceInterrupt {
                    interrupt: device_interrupt,
                })
            }
        };

        let mut interrupts = self.interrupts.lock().await;
        // NOTE(no-panic): index is in range of array
        let interrupt = interrupts.get_mut(index).unwrap();
        *interrupt = Some(pin);

        Ok(())
    }

    // FIXME: move this to Sensor (/!\ async, but fine, just need to always be used on a concrete
    // type directly)
    pub async fn wait_for_interrupt_event(
        &self,
        event: InterruptEvent,
    ) -> Result<(), InterruptError> {
        use lis3dh_async::{
            Interrupt1, InterruptConfig, InterruptMode, IrqPin1Config, Range, Threshold,
        };

        // FIXME: do something cleaner; return an error?
        // Wait for the driver to be initialized
        loop {
            if self.accel.lock().await.is_some() {
                break;
            }
            Timer::after(Duration::from_millis(10)).await;
        }

        match event {
            InterruptEvent {
                kind: InterruptEventKind::Accelerometer(AccelerometerInterruptEvent::LowG),
                duration,
            } => unimplemented!(),
            InterruptEvent {
                kind: InterruptEventKind::Accelerometer(AccelerometerInterruptEvent::FreeFall),
                duration,
            } => unimplemented!(),
            InterruptEvent {
                kind: InterruptEventKind::Accelerometer(AccelerometerInterruptEvent::Movement),
                duration,
            } => {
                // FIXME: should we lock the mutex for longer?
                let mut accel = self.accel.lock().await;

                // FIXME: don't disable other interrupts
                // FIXME: use either INT1 or INT2
                // Enable only INT1
                accel
                    .as_mut()
                    .unwrap()
                    .configure_interrupt_pin(IrqPin1Config {
                        ia1_en: true,
                        ..IrqPin1Config::default()
                    })
                    .await
                    .unwrap(); // FIXME: handle the error

                // FIXME: read this from the event
                let threshold = Threshold::g(Range::G2, 1.1);
                accel
                    .as_mut()
                    .unwrap()
                    .configure_irq_threshold(Interrupt1, threshold)
                    .await
                    .unwrap();

                if let Some(duration) = duration {
                    // NOTE(no-panic): the data_rate atomic is only written from a DataRate
                    let data_rate =
                        DataRate::try_from(self.data_rate.load(Ordering::Relaxed)).unwrap();
                    // Typo in library (miliseconds instead of milliseconds)
                    // FIXME: this does not seem to be working with other durations
                    let duration =
                        lis3dh_async::Duration::miliseconds(data_rate, duration.as_millis() as f32);
                    accel
                        .as_mut()
                        .unwrap()
                        .configure_irq_duration(Interrupt1, duration)
                        .await
                        .unwrap();
                }

                // TODO: copied from the docs, check this
                accel
                    .as_mut()
                    .unwrap()
                    .configure_irq_src(
                        Interrupt1,
                        InterruptMode::Movement,
                        InterruptConfig::high_and_low(),
                    )
                    .await
                    .unwrap(); // FIXME: handle the error
            }
            InterruptEvent {
                kind: InterruptEventKind::Accelerometer(AccelerometerInterruptEvent::Tap),
                ..
            } => unimplemented!(),
            InterruptEvent {
                kind: InterruptEventKind::Accelerometer(AccelerometerInterruptEvent::DoubleTap),
                ..
            } => unimplemented!(),
            InterruptEvent { kind, .. } => {
                return Err(InterruptError::UnsupportedInterruptEventKind { event_kind: kind });
            }
        }

        // FIXME: select the appropriate pin, return an error if no appropriate pin
        // Wait for the external interrupt to be triggered by the sensor device
        // FIXME: is it ok to keep the mutex locked for this long?
        self.interrupts
            .lock()
            .await
            .get_mut(0)
            .unwrap()
            .as_mut()
            .unwrap() // FIXME: check that a pin has been provided
            .wait_for_high()
            .await;

        Ok(())
    }
}

impl Sensor for Lis3dhI2c {
    fn trigger_measurement(&self) -> Result<(), MeasurementError> {
        if self.state.get() != State::Enabled {
            return Err(MeasurementError::NonEnabled);
        }

        // FIXME: clear/reset the `reading_channel`?

        self.trigger.signal(());

        Ok(())
    }

    fn wait_for_reading(&'static self) -> ReadingWaiter {
        if self.state.get() != State::Enabled {
            return ReadingWaiter::Err(ReadingError::NonEnabled);
        }

        self.reading_channel.receive().into()
    }

    fn available_interrupt_events(&self) -> &[InterruptEventKind] {
        use interrupts::AccelerometerInterruptEvent::*;

        &[
            InterruptEventKind::Accelerometer(LowG),
            InterruptEventKind::Accelerometer(FreeFall),
            InterruptEventKind::Accelerometer(Movement),
            InterruptEventKind::Accelerometer(Tap),
            InterruptEventKind::Accelerometer(DoubleTap),
        ]
    }

    fn set_mode(&self, mode: SensorMode) -> Result<State, ModeSettingError> {
        let new_state = self.state.set_mode(mode);

        if new_state == State::Uninitialized {
            Err(ModeSettingError::Uninitialized)
        } else {
            Ok(new_state)
        }
    }

    fn state(&self) -> State {
        self.state.get()
    }

    fn categories(&self) -> &'static [Category] {
        &[Category::Accelerometer]
    }

    fn reading_axes(&self) -> ReadingAxes {
        ReadingAxes::V3([
            ReadingAxis::new(Label::X, -2, PhysicalUnit::AccelG),
            ReadingAxis::new(Label::Y, -2, PhysicalUnit::AccelG),
            ReadingAxis::new(Label::Z, -2, PhysicalUnit::AccelG),
        ])
    }

    fn label(&self) -> Option<&'static str> {
        self.label
    }

    fn display_name(&self) -> Option<&'static str> {
        Some("3-axis accelerometer")
    }

    fn part_number(&self) -> Option<&'static str> {
        Some("LIS3DH")
    }

    fn version(&self) -> u8 {
        0
    }
}
