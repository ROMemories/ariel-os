//! Provides power management functionality.

use ariel_os_embassy_common::power::GpioWakeupTrigger;

pub use embassy_rp::gpio::{DormantWakeConfig, Pin};

/// Interrupts allowed to trigger a wake-up from standby mode.
#[derive(Debug, Default)]
pub struct WakeupInterrupts {}

#[doc(hidden)]
pub fn enter_stop_mode<'a, T: crate::IntoPeripheral<'a, P>, P: embassy_rp::gpio::Pin>(
    gpio_wakeup: Option<(T, ariel_os_embassy_common::gpio::Pull, GpioWakeupTrigger)>,
) {
    if let Some(w) = gpio_wakeup {
        let mut input = embassy_rp::gpio::Input::new(
            w.0.into_hal_peripheral(),
            crate::gpio::input::from_pull(w.1),
        );

        // TODO: support edges.
        let trigger = match w.2 {
            GpioWakeupTrigger::Low => DormantWakeConfig {
                level_low: true,
                ..Default::default()
            },
            GpioWakeupTrigger::High => DormantWakeConfig {
                level_high: true,
                ..Default::default()
            },
        };
        // Needs to be kept live for waking up.
        let _dormant_wake = input.dormant_wake(trigger);

        embassy_rp::clocks::dormant_sleep();
    } else {
        embassy_rp::clocks::dormant_sleep();
    }
}

#[doc(hidden)]
pub fn enter_standby_mode(interrupts: WakeupInterrupts) {
    todo!();
}
