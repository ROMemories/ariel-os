//! Provides power management functionality.

#![expect(unsafe_code)]

use ariel_os_embassy_common::power::GpioWakeupTriggerEvent;

// TODO: might want to use `ExtiPin` instead, but requires the `exti` Cargo feature
pub use embassy_stm32::gpio::Pin as StopWakeupPin;

/// Interrupts allowed to trigger a wake-up from standby mode.
#[derive(Debug, Default)]
pub struct WakeupInterrupts {
    /// Whether to allow waking up on external interrupts (these may be limited to a specific set
    /// of pins).
    pub gpio: bool,
    /// Whether to allow waking up on RTC events.
    pub rtc: bool,
}

#[doc(hidden)]
pub fn enter_stop_mode<'a, T: crate::IntoPeripheral<'a, P>, P: embassy_stm32::gpio::Pin>(
    gpio_wakeup: Option<(
        T,
        ariel_os_embassy_common::gpio::Pull,
        GpioWakeupTriggerEvent,
    )>,
) {
    // NOTE: a critical section is used for atomicity.
    critical_section::with(|_| {
        // TODO: use the Shutdown mode when `stm32-metapac` supports it.

        // NOTE: each Reference Manual gets its own branch.
        cfg_select! {
            // STM32C0: Table 28 of RM0490 Rev 5.
            context = "stm32c031c6" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STOP));
            }
            // STM32F0x2: Table 16 of TM0091 Rev 10.
            context = "stm32f042k6" => {
                use embassy_stm32::pac::pwr::vals::Pdds;

                embassy_stm32::pac::PWR.cr().modify(|w| w.set_pdds(Pdds::STOP_MODE));
            }
            // STM32F303: Table 20 of RM0316 Rev 10.
            any(context = "stm32f303cb", context = "stm32f303re") => {
                use embassy_stm32::pac::pwr::vals::Pdds;

                embassy_stm32::pac::PWR.cr().modify(|w| w.set_pdds(Pdds::STOP_MODE));
            }
            // STM32F401: Table 19 of RM0368 Rev 5.
            context = "stm32f401re" => {
                use embassy_stm32::pac::pwr::vals::Pdds;

                // The RM calls this register `PWR_CR`.
                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_pdds(Pdds::STOP_MODE));
            }
            // STM32F411: Table 18 of RM0383 Rev 3.
            context = "stm32f411re" => {
                use embassy_stm32::pac::pwr::vals::Pdds;

                // The RM calls this register `PWR_CR`.
                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_pdds(Pdds::STOP_MODE));
            }
            // STM32F76: Table 20 of RM0410 Rev 5.
            context = "stm32f767zi" => {
                use embassy_stm32::pac::pwr::vals::Pdds;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_pdds(Pdds::STOP_MODE));
            }
            // STM32H753: Table 39 and Table 46 of RM0433.
            context = "stm32h753zi" => {
                // FIXME.
            }
            // STM32H755: Table 40 and Table 47 of RM0399 Rev 4.
            context = "stm32h755zi" => {
                // FIXME.
            }
            // STM32L47: Table 30 of RM0351 Rev 10.
            context = "stm32l475vg" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STANDBY));
            }
            // STM32U0: Table 28 of RM0503 Rev 4.
            any(context = "stm32u073kc", context = "stm32u083mc") => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STOP2));
            }
            // STM32U5: Table 104 of RM0503 Rev 6.
            context = "stm32u585ai" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                // STOP3 only allow waking up from WKUP pins, so we use STOP2.
                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STOP2));
            }
            // STM32WB: Table 32 of RM0434 Rev 14.
            context = "stm32wb55rg" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STOP2));
            }
            // STM32WBA5: Table 92 of RM0493 Rev 7.
            context = "stm32wba55cg" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STOP1));
            }
            // STM32WBA6: Table 93 of RM0515 Rev 4.
            context = "stm32wba65ri" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                // FIXME: use the STOP2 mode when `stm32-metapac` supports it.
                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STOP1));
            }
            // STM32WLE: Table 45 for RM0461 Rev 10.
            context = "stm32wle5jc" => {
                use embassy_stm32::pac::pwr::vals::Lpms;

                embassy_stm32::pac::PWR.cr1().modify(|w| w.set_lpms(Lpms::STOP2));
            }
            _ => const { panic!("unsupported MCU") },
        }

        // SAFETY: the peripherals are obtained and used inside a single critical section.
        let mut p = unsafe { cortex_m::Peripherals::steal() };
        p.SCB.set_sleepdeep();
    });

    let gpio_wakeup_trigger = gpio_wakeup.as_ref().map(|w| w.2);

    // FIXME: set the proper pull value.
    let input = gpio_wakeup.map(|w| {
        let p = w.0.into_hal_peripheral();
        let port = p.port();
        let pin = p.pin();

        (crate::gpio::input::new(p, w.1, false), port, pin)
    });

    use embassy_stm32::pac::EXTI;

    fn cpu_regs() -> embassy_stm32::pac::exti::Exti {
        EXTI
    }

    fn exticr_regs() -> embassy_stm32::pac::exti::Exti {
        EXTI
    }

    if let Some(gpio_wakeup_trigger) = gpio_wakeup_trigger {
        // FIXME: these should be on edges, not states.
        let (rising, falling) = match gpio_wakeup_trigger {
            GpioWakeupTriggerEvent::Low => (false, true),
            GpioWakeupTriggerEvent::High => (true, false),
        };

        // TODO: refactor to avoid unwrapping.
        let port = input.as_ref().unwrap().1;
        let pin = input.as_ref().unwrap().2;

        critical_section::with(|_| {
            let pin = pin as usize;
            exticr_regs()
                .exticr(pin / 4)
                .modify(|w| w.set_exti(pin % 4, port));
            EXTI.rtsr(0).modify(|w| w.set_line(pin, rising));
            EXTI.ftsr(0).modify(|w| w.set_line(pin, falling));

            // Clear pending events.
            {
                EXTI.rpr(0).write(|w| w.set_line(pin, true));
                EXTI.fpr(0).write(|w| w.set_line(pin, true));
            }

            // Enabling *event* generation is necessary to wake-up from WFE.
            cpu_regs().emr(0).modify(|w| w.set_line(pin, true));
            // Enabling *interrupt* generation is necessary so that the interrupt flags are set, so
            // we can check which event woke us up after WFE completes.
            cpu_regs().imr(0).modify(|w| w.set_line(pin, true));
        });
    }

    embassy_stm32::pac::RCC
        .cfgr()
        .modify(|w| w.set_stopwuck(true));

    // TODO: is this needed?
    critical_section::with(|_| {
        let mut p = unsafe { cortex_m::Peripherals::steal() };
        p.SYST.disable_interrupt();
    });

    let mut lines = embassy_stm32::pac::exti::regs::Lines(0);

    critical_section::with(|_| {
        loop {
            // https://github.com/STMicroelectronics/stm32u0xx-hal-driver/blob/b2df6792633348d41cc549609a8611098c0e3798/Src/stm32u0xx_hal_pwr_ex.c#L431-L433
            cortex_m::asm::sev();
            cortex_m::asm::wfe();
            cortex_m::asm::wfe();

            // This is done inside the critical section to be sure no ISRs can reset the interrupt
            // flags before we read them.
            if let Some(gpio_wakeup_trigger) = gpio_wakeup_trigger {
                let pin = input.as_ref().unwrap().2;

                // FIXME: these should be on edges, not states.
                lines = match gpio_wakeup_trigger {
                    GpioWakeupTriggerEvent::Low => EXTI.fpr(0).read(),
                    GpioWakeupTriggerEvent::High => EXTI.rpr(0).read(),
                };

                if lines.line(pin.into()) {
                    break;
                }
            }
        }
    });

    // Clear the interrupt flags of GPIO lines.
    // This is done *after* the critical section so that ISRs potentially registered still have
    // access to the flags.
    {
        let bits = lines.0 & 0x0000_ffff;
        EXTI.rpr(0)
            .write_value(embassy_stm32::pac::exti::regs::Lines(bits));
        EXTI.fpr(0)
            .write_value(embassy_stm32::pac::exti::regs::Lines(bits));
    }

    critical_section::with(|_| {
        let mut p = unsafe { cortex_m::Peripherals::steal() };
        p.SCB.clear_sleepdeep();
    });

    // FIXME: reconfigure clocks.
}

#[doc(hidden)]
pub fn enter_standby_mode(interrupts: WakeupInterrupts) {
    // FIXME: set up the wake-up interrupts: external interrupts, RTC.

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
            // STM32F303: Table 21 of RM0316 Rev 10.
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
            _ => const { panic!("unsupported MCU") },
        }

        // SAFETY: the peripherals are obtained and used inside a single critical section.
        let mut p = unsafe { cortex_m::Peripherals::steal() };
        p.SCB.set_sleepdeep();
    });
}
