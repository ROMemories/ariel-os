//! Provides power management functionality.

/// Interrupts allowed to trigger a wake-up from standby mode.
#[derive(Debug, Default)]
pub struct WakeupInterrupts {
    /// Whether to allow waking up on external interrupts (these may be limited to a specific set
    /// of pins).
    pub gpio: bool,
}

#[doc(hidden)]
pub fn enter_stop_mode() {
}

#[doc(hidden)]
pub fn enter_standby_mode(interrupts: WakeupInterrupts) -> ! {
    todo!();
}
