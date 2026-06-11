//! Provides power management functionality.

/// Interrupts allowed to trigger a wake-up from standby mode.
#[derive(Debug, Default)]
pub struct WakeupInterrupts {}

#[doc(hidden)]
pub fn enter_stop_mode() {
    embassy_rp::clocks::dormant_sleep();
}

#[doc(hidden)]
pub fn enter_standby_mode(interrupts: WakeupInterrupts) {
    todo!();
}
