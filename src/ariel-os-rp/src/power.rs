//! Provides power management functionality.

use ariel_os_embassy_common::power::GpioWakeupTriggerEvent;

pub use embassy_rp::gpio::{DormantWakeConfig, Pin as StopWakeupPin};

#[doc(hidden)]
pub fn enter_stop_mode<'a, T: crate::IntoPeripheral<'a, P>, P: StopWakeupPin>(
    gpio_wakeup: Option<(
        T,
        ariel_os_embassy_common::gpio::Pull,
        GpioWakeupTriggerEvent,
    )>,
) {
    if let Some(w) = gpio_wakeup {
        let mut input = embassy_rp::gpio::Input::new(
            w.0.into_hal_peripheral(),
            crate::gpio::input::from_pull(w.1),
        );

        // TODO: support edges.
        let trigger = match w.2 {
            GpioWakeupTriggerEvent::Low => DormantWakeConfig {
                level_low: true,
                ..Default::default()
            },
            GpioWakeupTriggerEvent::High => DormantWakeConfig {
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
