//! Provides power management functionality.

#[doc(hidden)]
pub fn enter_stop_mode() {
    embassy_rp::clocks::dormant_sleep();
}
