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
pub fn enter_stop_mode(mut input: esp_hal::gpio::Input<'_>, wakeup: GpioWakeupTrigger) {
    let wakeup_source = esp_hal::rtc_cntl::sleep::GpioWakeupSource::new();

    let event = match wakeup {
        GpioWakeupTrigger::Low => esp_hal::gpio::WakeEvent::LowLevel,
        GpioWakeupTrigger::High => esp_hal::gpio::WakeEvent::HighLevel,
    };

    input.wakeup_enable(true, event).unwrap();

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
