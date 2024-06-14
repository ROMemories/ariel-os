use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use enum_iterator::Sequence;
use lis3dh_async::{Configuration, DataRate, Lis3dh as InnerLis3dh, Lis3dhI2C};
use portable_atomic::{AtomicU8, Ordering};
use riot_rs_embassy::{arch, Spawner};
use riot_rs_sensors::{
    sensor::{
        AccuracyError, Mode as SensorMode, ModeSettingError, PhysicalValue, PhysicalValues,
        ReadingError, ReadingInfo, ReadingInfos, ReadingResult, State,
    },
    Category, Label, PhysicalUnit, Sensor,
};

pub use crate::lis3dh_spi::Lis3dhSpi;
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
    state: AtomicU8,
    label: Option<&'static str>,
    // TODO: consider using MaybeUninit?
    accel: Mutex<CriticalSectionRawMutex, Option<InnerLis3dh<Lis3dhI2C<arch::i2c::I2cDevice>>>>,
    interrupts:
        Mutex<CriticalSectionRawMutex, [Option<arch::gpio::Input<'static>>; INTERRUPT_COUNT]>,
}

impl Lis3dhI2c {
    #[expect(clippy::new_without_default)]
    #[must_use]
    pub const fn new(label: Option<&'static str>) -> Self {
        // This is required to initialize the array because Input is not Copy.
        const NO_INTERRUPT: Option<arch::gpio::Input<'static>> = None;
        Self {
            state: AtomicU8::new(State::Uninitialized as u8),
            label,
            accel: Mutex::new(None),
            interrupts: Mutex::new([NO_INTERRUPT; INTERRUPT_COUNT]),
        }
    }

    pub fn init(
        &'static self,
        _spawner: Spawner,
        peripherals: Peripherals,
        i2c: arch::i2c::I2cDevice,
        config: Config,
    ) {
        if self.state.load(Ordering::Acquire) == State::Uninitialized as u8 {
            // TODO: can this be made shorter?
            let mut lis3dh_config = Configuration::default();
            lis3dh_config.mode = config.mode;
            lis3dh_config.datarate = config.datarate;
            lis3dh_config.enable_x_axis = config.enable_x_axis;
            lis3dh_config.enable_y_axis = config.enable_y_axis;
            lis3dh_config.enable_z_axis = config.enable_z_axis;
            lis3dh_config.block_data_update = config.block_data_update;
            lis3dh_config.enable_temperature = config.enable_temperature;

            // FIXME: add a timeout, blocks indefinitely if no device is connected
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

            let driver = embassy_futures::block_on(InnerLis3dh::new_i2c_with_config(
                i2c,
                config.address,
                lis3dh_config,
            ))
            .unwrap();

            // We use `try_lock()` instead of `lock()` to not make this function async.
            // This mutex cannot be locked at this point as it is private and can only be
            // locked when the sensor has been initialized successfully.
            let mut accel = self.accel.try_lock().unwrap();
            *accel = Some(driver);

            self.state.store(State::Enabled as u8, Ordering::Release);
        }
    }

    // FIXME: is this ok to make this method async?
    // FIXME: return an error
    pub async fn register_interrupt_pin(
        &self,
        pin: arch::gpio::Input<'static>,
        device_interrupt: DeviceInterrupt,
        event: InterruptEvent,
    ) {
        // FIXME: check first that the interrupt can indeed handle that kind of event

        let index = match device_interrupt {
            DeviceInterrupt::Int1 => 0,
            DeviceInterrupt::Int2 => 1,
            _ => todo!(), // FIXME: return an error
        };

        let mut interrupts = self.interrupts.lock().await;
        // NOTE(no-panic): index is in range of array
        let interrupt = interrupts.get_mut(index).unwrap();
        *interrupt = Some(pin);
    }

    // FIXME: move this to Sensor
    pub fn available_interrupt_events(&self) -> impl Iterator<Item = InterruptEventKind> {
        enum_iterator::all::<InterruptEventKind>()
    }

    // FIXME: return an error (e.g., no pin registered for this event)
    // FIXME: move this to Sensor (/!\ async)
    pub async fn wait_for_interrupt_event(&self, event: InterruptEvent) {
        use lis3dh_async::{
            Interrupt1, InterruptConfig, InterruptMode, IrqPin1Config, Range, Threshold,
        };

        // FIXME: do something cleaner
        // Wait for the driver to be initialized
        loop {
            if self.accel.lock().await.is_some() {
                break;
            }
            embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
        }

        match event {
            InterruptEvent {
                kind: InterruptEventKind::LowG,
            } => unimplemented!(),
            InterruptEvent {
                kind: InterruptEventKind::Movement,
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
        }

        // FIXME: select the appropriate pin
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
    }
}

// TODO: should this be a trait instead?
// TODO: add other variants if needed
#[derive(Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeviceInterrupt {
    Int0,
    Int1,
    Int2,
    Int3,
}

// FIXME: orientation
#[derive(Copy, Clone, PartialEq, Eq, Sequence)]
#[non_exhaustive]
pub enum InterruptEventKind {
    LowG,
    Movement,
}

#[derive(Copy, Clone, PartialEq, Eq)]
// TODO: make fields private?
pub struct InterruptEvent {
    pub kind: InterruptEventKind,
    // TODO: duration, thresholds
}

impl Sensor for Lis3dhI2c {
    #[allow(refining_impl_trait)]
    async fn measure(&self) -> ReadingResult<PhysicalValues> {
        if self.state.load(Ordering::Acquire) != State::Enabled as u8 {
            return Err(ReadingError::NonEnabled);
        }

        // TODO: maybe should check is_data_ready()?
        let data = self
            .accel
            .lock()
            .await
            .as_mut()
            .unwrap()
            .accel_norm()
            .await
            .map_err(|_| ReadingError::SensorAccess)?;

        #[allow(clippy::cast_possible_truncation)]
        // FIXME: dumb scaling, take precision into account
        // FIXME: specify the measurement error
        let x = PhysicalValue::new((data.x * 100.) as i32, AccuracyError::Unknown);
        let y = PhysicalValue::new((data.y * 100.) as i32, AccuracyError::Unknown);
        let z = PhysicalValue::new((data.z * 100.) as i32, AccuracyError::Unknown);

        Ok(PhysicalValues::V3([x, y, z]))
    }

    fn set_mode(&self, mode: SensorMode) -> Result<State, ModeSettingError> {
        if self.state.load(Ordering::Acquire) == State::Uninitialized as u8 {
            return Err(ModeSettingError::Uninitialized);
        }

        let state = State::from(mode);
        self.state.store(state as u8, Ordering::Release);
        Ok(state)
    }

    fn state(&self) -> State {
        let state = self.state.load(Ordering::Acquire);
        // NOTE(no-panic): the state atomic is only written from a State
        State::try_from(state).unwrap()
    }

    fn categories(&self) -> &'static [Category] {
        &[Category::Accelerometer]
    }

    fn reading_infos(&self) -> ReadingInfos {
        ReadingInfos::V3([
            ReadingInfo::new(Label::X, -2, PhysicalUnit::AccelG),
            ReadingInfo::new(Label::Y, -2, PhysicalUnit::AccelG),
            ReadingInfo::new(Label::Z, -2, PhysicalUnit::AccelG),
        ])
    }

    fn label(&self) -> Option<&'static str> {
        self.label
    }

    fn display_name(&self) -> Option<&'static str> {
        Some("3-axis accelerometer")
    }

    fn part_number(&self) -> &'static str {
        "LIS3DH"
    }

    fn version(&self) -> u8 {
        0
    }
}
