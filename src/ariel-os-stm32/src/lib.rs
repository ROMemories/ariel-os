//! Items specific to the STMicroelectronics STM32 MCUs.

#![no_std]
#![cfg_attr(nightly, feature(doc_auto_cfg))]
#![deny(missing_docs)]

pub mod gpio;

#[doc(hidden)]
pub mod peripheral {
    pub use embassy_stm32::Peripheral;
}

#[cfg(feature = "external-interrupts")]
#[doc(hidden)]
pub mod extint_registry;

#[cfg(feature = "i2c")]
pub mod i2c;

#[doc(hidden)]
pub mod identity;

#[cfg(feature = "spi")]
pub mod spi;

#[cfg(feature = "storage")]
#[doc(hidden)]
pub mod storage;

#[cfg(feature = "usb")]
#[doc(hidden)]
pub mod usb;

#[cfg(feature = "eth")]
#[doc(hidden)]
pub mod eth;

use embassy_stm32::Config;

#[doc(hidden)]
pub use embassy_stm32::{OptionalPeripherals, Peripherals, interrupt};

pub use embassy_stm32::peripherals;

#[cfg(feature = "executor-interrupt")]
pub(crate) use embassy_executor::InterruptExecutor as Executor;

#[cfg(feature = "hwrng")]
#[doc(hidden)]
pub mod hwrng;

#[cfg(feature = "executor-interrupt")]
include!(concat!(env!("OUT_DIR"), "/swi.rs"));

#[cfg(capability = "hw/stm32-dual-core")]
use {core::mem::MaybeUninit, embassy_stm32::SharedData};

// Ariel OS doesn't support the second core yet, but upstream needs this.
#[cfg(capability = "hw/stm32-dual-core")]
static SHARED_DATA: MaybeUninit<SharedData> = MaybeUninit::uninit();

#[cfg(feature = "executor-interrupt")]
#[doc(hidden)]
pub static EXECUTOR: Executor = Executor::new();

#[doc(hidden)]
#[must_use]
pub fn init() -> OptionalPeripherals {
    let mut config = Config::default();
    config.rcc = clock_config();

    #[cfg(not(capability = "hw/stm32-dual-core"))]
    let peripherals = embassy_stm32::init(config);

    #[cfg(capability = "hw/stm32-dual-core")]
    let peripherals = embassy_stm32::init_primary(config, &SHARED_DATA);

    OptionalPeripherals::from(peripherals)
}

fn clock_config() -> embassy_stm32::rcc::Config {
    #[cfg(not(feature = "rcc-config-override"))]
    {
        #[cfg(context = "st-nucleo-wb55")]
        {
            use embassy_stm32::rcc::*;

            let mut rcc = embassy_stm32::rcc::Config::default();

            rcc.hsi48 = Some(Hsi48Config {
                sync_from_usb: true,
            }); // needed for USB
            rcc.sys = Sysclk::PLL1_R;
            rcc.hse = Some(Hse {
                freq: embassy_stm32::time::Hertz(32000000),
                mode: HseMode::Oscillator,
                prescaler: HsePrescaler::DIV1,
            });
            rcc.pll = Some(Pll {
                source: PllSource::HSE,
                prediv: PllPreDiv::DIV2,
                mul: PllMul::MUL10,
                divp: None,
                divq: None,
                divr: Some(PllRDiv::DIV2), // sysclk 80Mhz (32 / 2 * 10 / 2)
            });
            rcc.mux.clk48sel = mux::Clk48sel::HSI48;

            rcc
        }

        #[cfg(context = "st-nucleo-f767zi")]
        {
            use embassy_stm32::rcc::*;
            config.rcc.hse = Some(Hse {
                freq: embassy_stm32::time::Hertz(8000000),
                mode: HseMode::Bypass,
            });
            config.rcc.pll_src = PllSource::HSE;
            config.rcc.pll = Some(Pll {
                prediv: PllPreDiv::DIV4,
                mul: PllMul::MUL216,
                divp: Some(PllPDiv::DIV2),
                divq: None,
                divr: None,
            });
            config.rcc.ahb_pre = AHBPrescaler::DIV1;
            config.rcc.apb1_pre = APBPrescaler::DIV4;
            config.rcc.apb2_pre = APBPrescaler::DIV2;
            config.rcc.sys = Sysclk::PLL1_P;
        }

        #[cfg(context = "stm32h755zi")]
        {
            use embassy_stm32::rcc::*;

            let mut rcc = embassy_stm32::rcc::Config::default();

            rcc.hsi = Some(HSIPrescaler::DIV1);
            rcc.csi = true;
            rcc.hsi48 = Some(Hsi48Config {
                sync_from_usb: true,
            }); // needed for USB
            rcc.pll1 = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV4,
                mul: PllMul::MUL50,
                divp: Some(PllDiv::DIV2),
                // Required for SPI (configured by `spi123sel`)
                divq: Some(PllDiv::DIV16), // FIXME: adjust this divider
                divr: None,
            });
            rcc.sys = Sysclk::PLL1_P; // 400 Mhz
            rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
            rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
            rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
            rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
            rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
            rcc.voltage_scale = VoltageScale::Scale1;
            // Set SMPS power config otherwise MCU will not powered after next power-off
            rcc.supply_config = SupplyConfig::DirectSMPS;
            rcc.mux.usbsel = mux::Usbsel::HSI48;
            // Select the clock signal used for SPI1, SPI2, and SPI3.
            // FIXME: what to do about SPI4, SPI5, and SPI6?
            rcc.mux.spi123sel = mux::Saisel::PLL1_Q; // Reset value

            rcc
        }

        #[cfg(context = "stm32u083mc")]
        {
            use embassy_stm32::rcc::*;

            config.rcc.hsi48 = Some(Hsi48Config {
                sync_from_usb: true,
            }); // needed for USB
            // No HSE fitted on the stm32u083c-dk board
            config.rcc.hsi = true;
            config.rcc.sys = Sysclk::PLL1_R;
            config.rcc.pll = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV1,
                mul: PllMul::MUL7,
                divp: None,
                divq: None,
                divr: Some(PllRDiv::DIV2), // sysclk 56Mhz
            });
            config.rcc.mux.clk48sel = mux::Clk48sel::HSI48;
        }
    }

    #[cfg(feature = "rcc-config-override")]
    {
        unsafe extern "Rust" {
            fn __ariel_os_rcc_config() -> embassy_stm32::rcc::Config;
        }
        unsafe { __ariel_os_rcc_config() }
    }

    // mark used
    let _ = config;
}
