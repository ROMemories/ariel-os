use embassy_time::Duration;

// TODO: should this be a trait instead?
// TODO: add other variants if needed
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DeviceInterrupt {
    Int0,
    Int1,
    Int2,
    Int3,
}

impl core::fmt::Display for DeviceInterrupt {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Int0 => write!(f, "INT0"),
            Self::Int1 => write!(f, "INT1"),
            Self::Int2 => write!(f, "INT2"),
            Self::Int3 => write!(f, "INT3"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum InterruptEventKind {
    Accelerometer(AccelerometerInterruptEvent),
}

impl core::fmt::Display for InterruptEventKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Accelerometer(event) => write!(f, "{}", event),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
// FIXME: orientation
pub enum AccelerometerInterruptEvent {
    /// Acceleration values are below a given threshold on all axis.
    LowG,
    /// Same as [`LowG`](InterruptEventKind::LowG), but thresholds are provided by the sensor
    /// driver.
    /// Only available on 3-axis accelerometers.
    FreeFall,
    Movement,
    Tap,
    DoubleTap,
}

impl core::fmt::Display for AccelerometerInterruptEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::LowG => write!(f, "low g"),
            Self::FreeFall => write!(f, "free fall"),
            Self::Movement => write!(f, "movement"),
            Self::Tap => write!(f, "tap"),
            Self::DoubleTap => write!(f, "double tap"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
// TODO: make fields private?
pub struct InterruptEvent {
    pub kind: InterruptEventKind,
    pub duration: Option<Duration>,
    // pub threshold: Option<>, // FIXME
}

#[derive(Debug)]
pub enum InterruptError {
    UnsupportedDeviceInterrupt { interrupt: DeviceInterrupt },
    UnsupportedInterruptEventKind { event_kind: InterruptEventKind },
}

impl core::fmt::Display for InterruptError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsupportedDeviceInterrupt { interrupt } => write!(
                f,
                "device interrupt `{}` is not supported for sensor",
                interrupt,
            ),
            Self::UnsupportedInterruptEventKind { event_kind } => write!(
                f,
                "interrupt event `{}` is not supported for sensor",
                event_kind,
            ),
        }
    }
}

impl core::error::Error for InterruptError {}
