//! Provides power management functionality.

pub trait StopWakeupPin {}

/// Interrupts to configure to trigger a wake-up from standby mode.
#[derive(Debug, Default)]
pub struct WakeupInterrupts {
    _private: (),
}

pub fn enter_stop_mode() {
    unimplemented!();
}

pub fn enter_standby_mode(interrupts: WakeupInterrupts) {
    unimplemented!();
}
