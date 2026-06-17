//! Provides power management functionality.

#![allow(unsafe_code)]

use ariel_os_embassy_common::power::GpioWakeupTrigger;

/// Interrupts allowed to trigger a wake-up from standby mode.
#[derive(Debug, Default)]
pub struct WakeupInterrupts {
    /// Whether to allow waking up on external interrupts (these may be limited to a specific set
    /// of pins).
    pub gpio: bool,
}

#[doc(hidden)]
pub fn enter_stop_mode(gpio_wakeup: Option<(esp_hal::gpio::Input<'_>, GpioWakeupTrigger)>) {
    if let Some(mut gpio_wakeup) = gpio_wakeup {
        let event = match gpio_wakeup.1 {
            GpioWakeupTrigger::Low => esp_hal::gpio::WakeEvent::LowLevel,
            GpioWakeupTrigger::High => esp_hal::gpio::WakeEvent::HighLevel,
        };

        gpio_wakeup.0.wakeup_enable(true, event).unwrap();
    }

    let wakeup_source = esp_hal::rtc_cntl::sleep::GpioWakeupSource::new();

    critical_section::with(|_| {
        // SAFETY: the peripheral is stolen and used entirely in a critical section.
        let lpwr = unsafe { esp_hal::peripherals::LPWR::steal() };
        let mut rtc = esp_hal::rtc_cntl::Rtc::new(lpwr);
        rtc.sleep_light(&[&wakeup_source]);
    });
}

#[doc(hidden)]
pub fn enter_standby_mode(interrupts: WakeupInterrupts) {
    todo!();
}
