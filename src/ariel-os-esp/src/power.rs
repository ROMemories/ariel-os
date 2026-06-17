//! Provides power management functionality.

#![allow(unsafe_code)]

/// Interrupts allowed to trigger a wake-up from standby mode.
#[derive(Debug, Default)]
pub struct WakeupInterrupts {
    /// Whether to allow waking up on external interrupts (these may be limited to a specific set
    /// of pins).
    pub gpio: bool,
}

#[doc(hidden)]
pub fn enter_stop_mode(mut input: esp_hal::gpio::Input<'_>) {
    let wakeup_source = esp_hal::rtc_cntl::sleep::GpioWakeupSource::new();

    // TODO: check unwrap;
    input.wakeup_enable(true, esp_hal::gpio::WakeEvent::LowLevel).unwrap();

    critical_section::with(|_| {
        let lpwr = unsafe { esp_hal::peripherals::LPWR::steal() };
        let mut rtc = esp_hal::rtc_cntl::Rtc::new(lpwr);
        rtc.sleep_light(&[&wakeup_source]);
    });
}

#[doc(hidden)]
pub fn enter_standby_mode(interrupts: WakeupInterrupts) {
    todo!();
}
