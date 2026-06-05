//! Provides power management functionality.

#![deny(missing_docs)]
#![cfg_attr(not(context = "native"), no_std)]

mod reset;

use ariel_os_hal::hal::power::WakeupInterrupts;

pub use reset::*;

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
pub fn enter_standby_mode(interrupts: WakeupInterrupts) -> ! {
    cfg_select! {
        context = "nrf" => {
            ariel_os_hal::hal::power::enter_standby_mode(interrupts);
        }
        context = "stm32" => {
            enter_standby_mode_stm32()
        }
        _ => {
            let _ = interrupts;
        }
    }

    // This loop will not be executed, this is only to satisfy the return type.
    #[allow(clippy::empty_loop, reason = "for platform-independent tooling only")]
    loop {
        cfg_select! {
            context = "cortex-m" => cortex_m::asm::wfi(),
            _ => {}
        }
    }
}

#[cfg(context = "stm32")]
fn enter_standby_mode_stm32() -> ! {
    #![allow(unsafe_code)]

    // NOTE: a critical section is used for atomicity.
    critical_section::with(|_| {
        // TODO: use the Shutdown mode when `stm32-metapac` supports it.

        // NOTE: each Reference Manual gets its own branch.
        cfg_select! {
            // STM32C0: Table 28 of RM0490 Rev 5.
            context = "stm32c031c6" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STANDBY));
            }
            // STM32F0x2: Table 17 of TM0091 Rev 10.
            context = "stm32f042k6" => {
                use embassy_stm32::pac::pwr::vals::Pdds;

                embassy_stm32::pac::PWR.cr().modify(|w| w.set_pdds(Pdds::STANDBY_MODE));
            }
            // STM32F303: Table 20 of RM0316 Rev 10.
            any(context = "stm32f303cb", context = "stm32f303re") => {
                use embassy_stm32::pac::pwr::vals::Pdds;

                embassy_stm32::pac::PWR.cr().modify(|w| w.set_pdds(Pdds::STANDBY_MODE));
            }
            // STM32F401: Table 20 of RM0368 Rev 5.
            context = "stm32f401re" => {
                use embassy_stm32::pac::pwr::vals::Pdds;

                // The RM calls this register `PWR_CR`.
                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_pdds(Pdds::STANDBY_MODE));
            }
            // STM32F411: Table 19 of RM0383 Rev 3.
            context = "stm32f411re" => {
                use embassy_stm32::pac::pwr::vals::Pdds;

                // The RM calls this register `PWR_CR`.
                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_pdds(Pdds::STANDBY_MODE));
            }
            // STM32F76: Table 21 of RM0410 Rev 5.
            context = "stm32f767zi" => {
                use embassy_stm32::pac::pwr::vals::Pdds;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_pdds(Pdds::STANDBY_MODE));
            }
            // STM32H753: Table 39 and Table 46 of RM0433.
            context = "stm32h753zi" => {
                // FIXME: needs some power measurements to confirm this works as expected.
                embassy_stm32::pac::PWR.cpucr().modify(|w| w.set_run_d3(false));

                embassy_stm32::pac::PWR.cpucr().modify(|w| w.set_pdds_d1(true));
                embassy_stm32::pac::PWR.cpucr().modify(|w| w.set_pdds_d2(true));
                embassy_stm32::pac::PWR.cpucr().modify(|w| w.set_pdds_d3(true));
            }
            // STM32H755: Table 40 and Table 47 of RM0399 Rev 4.
            context = "stm32h755zi" => {
                // FIXME: needs some power measurements to confirm this works as expected.
                embassy_stm32::pac::PWR.cpucr().modify(|w| w.set_run_d3(false));

                embassy_stm32::pac::PWR.cpucr().modify(|w| w.set_pdds_d1(true));
                embassy_stm32::pac::PWR.cpucr().modify(|w| w.set_pdds_d2(true));
                embassy_stm32::pac::PWR.cpucr().modify(|w| w.set_pdds_d3(true));
            }
            // STM32L47: Table 30 of RM0351 Rev 10.
            context = "stm32l475vg" => {
                use embassy_stm32::pac::pwr::vals::{Lpms, Rrs};

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STANDBY));
                embassy_stm32::pac::PWR.cr3().modify(|w| w.set_rrs(Rrs::POWER_OFF));
            }
            // STM32U0: Table 29 of RM0503 Rev 4.
            any(context = "stm32u073kc", context = "stm32u083mc") => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STANDBY));
                // Lose SRAM2 contents.
                embassy_stm32::pac::PWR.cr3().modify(|w| w.set_rrs(false));
            }
            // STM32U5: Table 105 of RM0503 Rev 6.
            context = "stm32u585ai" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                // TODO: use the Standby/Shutdown mode when `stm32-metapac` supports it.
                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STOP3));
                // Lose SRAM2 contents.
                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_rrsb1(false));
                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_rrsb2(false));
            }
            // STM32WB: Table 33 of RM0434 Rev 14.
            context = "stm32wb55rg" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STANDBY));
                embassy_stm32::pac::PWR.cr3().modify(|w| w.set_rrs(false));
            }
            // STM32WBA5: Table 94 of RM0493 Rev 7.
            context = "stm32wba55cg" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                // FIXME: use the Standby/Shutdown mode when `stm32-metapac` supports it.
                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STOP1));
            }
            // STM32WBA6: Table 96 of RM0515 Rev 4.
            context = "stm32wba65ri" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                // FIXME: use the Standby/Shutdown mode when `stm32-metapac` supports it.
                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STOP1));
            }
            // STM32WLE: Table 46 for RM0461 Rev 10.
            context = "stm32wle5jc" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STANDBY));
                embassy_stm32::pac::PWR.cr3().modify(|w| w.set_rrs(false));
            }
        }

        // SAFETY: the peripherals are obtained and used inside a single critical section.
        let mut p = unsafe { cortex_m::Peripherals::steal() };
        p.SCB.set_sleepdeep();
    });
}
