//! Provides power management functionality.

#![deny(missing_docs)]
#![cfg_attr(not(context = "native"), no_std)]

mod reset;

use ariel_os_hal::{gpio::Pull, hal::power::WakeupInterrupts};

pub use ariel_os_embassy_common::power::GpioWakeupTriggerEvent;
pub use reset::*;

/// FIXME
pub struct GpioStopWakeupTrigger<
    'a,
    T: ariel_os_hal::hal::IntoPeripheral<'a, P>,
    P: ariel_os_hal::hal::power::Pin,
> {
    /// FIXME
    pub gpio: T,
    /// FIXME
    pub pull: ariel_os_hal::gpio::Pull,
    /// FIXME
    pub event: GpioWakeupTriggerEvent,
    _phantom: core::marker::PhantomData<&'a P>,
}

impl<'a, T: ariel_os_hal::hal::IntoPeripheral<'a, P>, P: ariel_os_hal::hal::power::Pin>
    GpioStopWakeupTrigger<'a, T, P>
{
    /// Creates a  to define on which event to wake up from
    /// [stop mode](enter_stop_mode).
    #[must_use]
    pub fn new(gpio: T, pull: Pull, event: GpioWakeupTriggerEvent) -> Self {
        Self {
            gpio,
            pull,
            event,
            _phantom: core::marker::PhantomData,
        }
    }
}

/// Interrupts and events allowed to trigger a wake-up from stop mode.
#[non_exhaustive]
pub struct StopWakeupTriggers<
    'a,
    T: ariel_os_hal::hal::IntoPeripheral<'a, P>,
    P: ariel_os_hal::hal::power::Pin,
> {
    /// Whether to allow waking up on external interrupts (these may be limited to a specific set
    /// of pins).
    pub gpio: Option<GpioStopWakeupTrigger<'a, T, P>>,
    pub(crate) _phantom: core::marker::PhantomData<&'a P>,
}

impl<'a, T: ariel_os_hal::hal::IntoPeripheral<'a, P>, P: ariel_os_hal::hal::power::Pin> Default
    for StopWakeupTriggers<'a, T, P>
{
    fn default() -> Self {
        Self {
            gpio: None,
            _phantom: core::marker::PhantomData,
        }
    }
}

/// Enters stop mode.
///
/// In this mode, almost every clock of the microcontroller is off, but the RAM contents are
/// retained.
/// Unlike [`enter_standby_mode()`], waking up does not involve rebooting, and execution resumes
/// normally after calling this function.
///
/// # Important note
///
/// This is currently implemented on a best-effort basis.
/// Some microcontrollers may not support these low-power settings, they may not be implemented
/// yet, or they may be lacking testing.
/// Do measure the power consumption of your hardware when relevant for your application.
///
/// # Wake-up conditions
///
/// Depending on the microcontroller, waking up from this mode usually requires an RTC interrupt or
/// an external interrupt (sometimes on a limited set of pins).
#[cfg(feature = "lp-modes")]
pub fn enter_stop_mode<
    'a,
    T: ariel_os_hal::hal::IntoPeripheral<'a, P>,
    P: ariel_os_hal::hal::power::Pin,
>(
    wakeup: StopWakeupTriggers<'a, T, P>,
) {
    match wakeup {
        StopWakeupTriggers {
            gpio: Some(gpio), ..
        } => ariel_os_hal::hal::power::enter_stop_mode(Some((gpio.gpio, gpio.pull, gpio.event))),
        StopWakeupTriggers { gpio: None, .. } => {
            ariel_os_hal::hal::power::enter_stop_mode::<T, _>(None)
        }
    }
}

/// Enters standby mode.
///
/// In this mode, almost every clock of the microcontroller is off, and the RAM is powered off when
/// possible, requiring rebooting the application completely when waking up.
/// This function never returns to represent that.
///
/// # Important note
///
/// This is currently implemented on a best-effort basis.
/// Some microcontrollers may not support these low-power settings, they may not be implemented
/// yet, or they may be lacking testing.
/// Do measure the power consumption of your hardware when relevant for your application.
///
/// # Wake-up conditions
///
/// The conditions allowing to trigger a wake-up depend on the hardware.
/// The `WakeupInterrupts` type is used to configure which interrupts can trigger a wake-up.
/// This type is HAL-specific and can be found in `ariel_os::hal::power`.
/// On some hardware, it may however not be possible to prevent specific interrupts from triggering
/// a wake-up.
///
/// If the hardware does not support triggering a reset on wake-up, or if all the wake-up
/// conditions are disabled through `WakeupInterrupts`, this function functionally powers down the
/// microcontroller, with no ability to automatically wake up (except from a hardware reset).
#[cfg(feature = "lp-modes")]
pub fn enter_standby_mode(interrupts: WakeupInterrupts) -> ! {
    ariel_os_hal::hal::power::enter_standby_mode(interrupts);

    // This loop will not be executed, this is only to satisfy the return type.
    #[allow(clippy::empty_loop, reason = "for platform-independent tooling only")]
    loop {
        cfg_select! {
            context = "cortex-m" => cortex_m::asm::wfi(),
            _ => {}
        }
    }
}
