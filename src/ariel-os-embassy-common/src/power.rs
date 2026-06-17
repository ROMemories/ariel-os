//! Provides power management functionality.

/// GPIO event that should trigger a wake-up.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GpioWakeupTrigger {
    /// The GPIO is low.
    Low,
    /// The GPIO is high.
    High,
}
