//! Provides power management functionality.

/// Interrupts allowed to trigger a wake-up from standby mode.
#[derive(Debug, Default)]
pub struct WakeupInterrupts {
    /// Whether to allow waking up on external interrupts (these may be limited to a specific set
    /// of pins).
    pub gpio: bool,
}

#[doc(hidden)]
pub fn enter_stop_mode() {
    // Nothing to do: the PMU should automatically feature gate peripherals as appropriate.
}

#[doc(hidden)]
pub fn enter_standby_mode(interrupts: WakeupInterrupts) {
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
                _ => panic!("unsupported MCU"),
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

        embassy_nrf::pac::POWER
            .systemoff()
            .write(|w| w.set_systemoff(true));
    });

    cortex_m::asm::wfi();
}

// Requires a critical section to guarantee atomicity of the sequence of operations.
fn disable_sense_nrf(_cs: critical_section::CriticalSection<'_>) {
    use embassy_nrf::pac;

    let ports: &[(_, usize)] = cfg_select! {
        context = "nrf51822-xxaa" => todo!(),
        context = "nrf52832" => todo!(),
        context = "nrf52833" => todo!(),
        context = "nrf52840" => &[(pac::P0, 32), (pac::P1, 16)],
        context = "nrf5340-app" => todo!(),
        context = "nrf5340-net" => todo!(),
        any(context = "nrf9151", context = "nrf9160") => todo!(),
        _ => const { panic!("unsupported MCU") },
    };

    for (port, pin_count) in ports {
        for pin in 0..*pin_count {
            port.pin_cnf(pin)
                .modify(|w| w.set_sense(embassy_nrf::pac::gpio::vals::Sense::DISABLED));
        }
    }

    // Clear `EVENTS_PORTS` (see section 6.10.2 of the nRF52840 datasheet v1.8).
    pac::GPIOTE.events_port().write_value(0);
}
