//! Provides MCU-specific items.
//!
//! This module dispatches between one of the following crate, depending on the target
//! architecture:
//!
//! | Manufacturer         | MCU family  | Items imported                                       |
//! | -------------------- | ----------- | ---------------------------------------------------- |
//! | Espressif            | ESP32       | [`riot-rs-esp::*`](../../riot_rs_esp/index.html)     |
//! | Nordic Semiconductor | nRF         | [`riot-rs-nrf::*`](../../riot_rs_nrf/index.html)     |
//! | Raspberry Pi         | RP          | [`riot-rs-rp::*`](../../riot_rs_rp/index.html)       |
//! | STMicroelectronics   | STM32       | [`riot-rs-stm32::*`](../../riot_rs_stm32/index.html) |

#![no_std]

cfg_if::cfg_if! {
    if #[cfg(context = "nrf")] {
        pub use riot_rs_nrf::*;
    } else if #[cfg(context = "rp")] {
        pub use riot_rs_rp::*;
    } else if #[cfg(context = "esp")] {
        pub use riot_rs_esp::*;
    } else if #[cfg(context = "stm32")] {
        pub use riot_rs_stm32::*;
    } else if #[cfg(context = "riot-rs")] {
        compile_error!("this architecture is not supported");
    } else {
        mod dummy;
        pub use dummy::*;
    }
}
