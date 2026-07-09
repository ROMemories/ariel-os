//! Provides power management functionality.

#![allow(unsafe_code)]

use ariel_os_embassy_common::power::GpioWakeupTriggerEvent;

pub use esp_hal::gpio::InputPin as Pin;

/// Interrupts allowed to trigger a wake-up from standby mode.
#[derive(Debug, Default)]
pub struct WakeupInterrupts {
    /// Whether to allow waking up on external interrupts (these may be limited to a specific set
    /// of pins).
    pub gpio: bool,
}

#[doc(hidden)]
pub fn enter_stop_mode<'a, T: crate::IntoPeripheral<'a, P>, P: esp_hal::gpio::InputPin>(
    gpio_wakeup: Option<(
        T,
        ariel_os_embassy_common::gpio::Pull,
        GpioWakeupTriggerEvent,
    )>,
) {
    let input = gpio_wakeup.map(|w| {
        let event = match w.2 {
            GpioWakeupTriggerEvent::Low => esp_hal::gpio::WakeEvent::LowLevel,
            GpioWakeupTriggerEvent::High => esp_hal::gpio::WakeEvent::HighLevel,
        };

        let config =
            esp_hal::gpio::InputConfig::default().with_pull(crate::gpio::input::from_pull(w.1));
        let mut input = esp_hal::gpio::Input::new(w.0.into_hal_peripheral(), config);
        input.wakeup_enable(true, event).unwrap();

        input
    });

    let wakeup_source = esp_hal::rtc_cntl::sleep::GpioWakeupSource::new();

    critical_section::with(|_| {
        // SAFETY: the peripheral is stolen and used entirely in a critical section.
        let lpwr = unsafe { esp_hal::peripherals::LPWR::steal() };
        let mut rtc = esp_hal::rtc_cntl::Rtc::new(lpwr);
        rtc.sleep_light(&[&wakeup_source]);
    });

    drop(input);
}

#[doc(hidden)]
pub fn enter_standby_mode(interrupts: WakeupInterrupts) {
    todo!();
}
