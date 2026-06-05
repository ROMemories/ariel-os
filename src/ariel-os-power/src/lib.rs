//! Provides power management functionality.

#![deny(missing_docs)]
#![cfg_attr(not(context = "native"), no_std)]

mod reset;

pub use reset::*;

/// Interrupts to configure to trigger a wake-up from [standby mode](enter_standby_mode()).
#[derive(Debug, Default)]
pub struct WakeupInterrupts {
    /// Allow waking up on external interrupts (these may be limited to a specific set of pins).
    pub gpio: bool,
    /// Allow waking up on an RTC event.
    #[cfg(context = "stm32")]
    pub rtc: bool,
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
/// Depending on the microcontroller, waking up from this mode usually requires an RTC interrupt or
/// an external interrupt (sometimes on a limited set of pins).
pub fn enter_standby_mode(interrupts: WakeupInterrupts) -> ! {
    cfg_select! {
        context = "nrf" => {
            enter_standby_mode_nrf(interrupts)
        }
        context = "stm32" => {
            enter_standby_mode_stm32()
        }
        _ => {
            let _ = interrupts;

            #[expect(clippy::empty_loop, reason = "for platform-independent tooling only")]
            loop {}
        }
    }
}

#[cfg(context = "nrf")]
fn enter_standby_mode_nrf(interrupts: WakeupInterrupts) -> ! {
    cfg_select! {
        context = "nrf51822-xxaa" => {
            embassy_nrf::pac::POWER.ramon().modify(|w| w.set_offram0(embassy_nrf::pac::power::vals::Offram0::RAM0OFF));
            embassy_nrf::pac::POWER.ramon().modify(|w| w.set_offram1(embassy_nrf::pac::power::vals::Offram1::RAM1OFF));
            embassy_nrf::pac::POWER.ramonb().modify(|w| w.set_offram2(embassy_nrf::pac::power::vals::Offram2::RAM2OFF));
            embassy_nrf::pac::POWER.ramonb().modify(|w| w.set_offram3(embassy_nrf::pac::power::vals::Offram3::RAM3OFF));
        }
        _ => {
            const RAM_BLOCK_COUNT: usize = cfg_select! {
                context = "nrf52832" => 8,
                context = "nrf52833" => 9,
                context = "nrf52840" => 9,
                context = "nrf5340-app" => 8,
                context = "nrf5340-net" => 4,
                any(context = "nrf9151", context = "nrf9160") => 8,
            };

            let (peripheral, value) = cfg_select! {
                any(context = "nrf53", context = "nrf91") => {
                    (embassy_nrf::pac::VMC, embassy_nrf::pac::vmc::regs::Power(0xffff_0000))
                }
                _ => (embassy_nrf::pac::POWER, embassy_nrf::pac::power::regs::Power(0xffff_0000)),
            };

            // Make sure the retention of every RAM section is disabled in *System OFF* mode.
            // See Table 17 of the nRF52840 datasheet v1.8.
            for i in 0..RAM_BLOCK_COUNT {
                peripheral.ram(i).powerclr().write_value(value);
            }
        }
    }

    critical_section::with(|cs| {
        // If external interrupts should not trigger a wake-up.
        if !interrupts.gpio {
            disable_sense_nrf(cs);
        }

        embassy_nrf::pac::POWER.systemoff().write(|w| w.set_systemoff(true));
    });

    // This loop will not be executed, this is only to satisfy the return type.
    loop {
        cortex_m::asm::wfi();
    }
}

// Requires a critical section to guarantee atomicity of the sequence of operations.
#[cfg(context = "nrf")]
fn disable_sense_nrf(_cs: critical_section::CriticalSection) {
    use embassy_nrf::pac;

    let ports: &[(_, usize)] = cfg_select! {
        context = "nrf51822-xxaa" => todo!(),
        context = "nrf52832" => todo!(),
        context = "nrf52833" => todo!(),
        context = "nrf52840" => &[(pac::P0, 32), (pac::P1, 16)],
        context = "nrf5340-app" => todo!(),
        context = "nrf5340-net" => todo!(),
        any(context = "nrf9151", context = "nrf9160") => todo!(),
        _ => panic!("unsupported MCU"),
    };

    for (port, pin_count) in ports {
        for pin in 0..*pin_count {
            port.pin_cnf(pin).modify(|w| w.set_sense(embassy_nrf::pac::gpio::vals::Sense::DISABLED));
        }
    }

    // Clear `EVENTS_PORTS` (see section 6.10.2 of the nRF52840 datasheet v1.8).
    pac::GPIOTE.events_port().write_value(0);
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

    // A single iteration of this loop will be executed, but this satisfies the return
    // type.
    loop {
        cortex_m::asm::wfi();
    }
}
