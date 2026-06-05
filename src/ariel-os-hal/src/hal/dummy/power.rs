//! Provides power management functionality.

/// Interrupts to configure to trigger a wake-up from standby mode.
#[derive(Debug, Default)]
pub struct WakeupInterrupts {
    _private: (),
}

pub fn enter_standby_mode(interrupts: WakeupInterrupts) {
    unimplemented!();
}
