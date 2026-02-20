//! Provides power management functionality.

#![deny(missing_docs)]
#![cfg_attr(not(context = "native"), no_std)]

/// Reboots the MCU.
///
/// This function initiates a software reset of the microcontroller and never returns.
pub fn reboot() -> ! {
    cfg_if::cfg_if! {
        if #[cfg(context = "cortex-m")] {
            cortex_m::peripheral::SCB::sys_reset()
        } else if #[cfg(context = "esp")] {
            esp_hal::system::software_reset()
        } else if #[cfg(context = "native")] {
            std::process::exit(0)
        } else if #[cfg(context = "ariel-os")] {
            compile_error!("reboot is not yet implemented for this platform")
        } else {
            #[expect(clippy::empty_loop, reason = "for platform-independent tooling only")]
            loop {}
        }
    }
}

/// Enters shutdown mode.
///
/// In this mode, almost every clock of the microcontroller is off, and the RAM contents may or
/// may not be retained, requiring rebooting the application completely when waking-up.
/// This function never returns to represent that.
///
/// # Wake-up conditions
///
/// Depending on the microcontroller, waking-up usually requires an RTT/RTC interrupt or an
/// external interrupt (sometimes on a limited set of pins).
pub fn enter_shutdown_mode() -> ! {
    // TODO: split this into HAL-specific modules
    #![allow(unsafe_code, reason = "only for STM32")]

    cfg_if::cfg_if! {
        if #[cfg(context = "stm32")] {
            use embassy_stm32::pac::pwr::vals::Lpms;

            // TODO: maybe use a critical section?

            // FIXME: stm32_metapac does not seem to support shutdown
            embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STANDBY));

            // TODO: safety comment
            let mut p = unsafe { cortex_m::Peripherals::steal() };
            p.SCB.set_sleepdeep();

            // A single iteration of this loop will be executed, but this satisfies the return
            // type.
            loop {
                cortex_m::asm::wfi();
            }
        } else {
            // TODO: use WFI
            #[expect(clippy::empty_loop, reason = "for platform-independent tooling only")]
            loop {}
        }
    }
}

/// Enters dormant mode.
///
/// In this mode, almost every clock of the microcontroller is off, but the RAM contents are
/// retained.
///
/// # Wake-up conditions
///
/// Depending on the microcontroller, waking-up usually requires an RTT/RTC interrupt or an
/// external interrupt (sometimes on a limited set of pins).
pub fn enter_dormant_mode() {
    todo!();
}
