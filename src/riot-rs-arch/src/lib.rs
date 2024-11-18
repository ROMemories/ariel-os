//! Provides MCU-specific items.
//!
//! This module dispatches between one of the following crate, depending on the target
//! architecture:
//!
//! | Manufacturer         | MCU family  | Docs rendered for | Items imported                                       |
//! | -------------------- | ----------- | ----------------- | ---------------------------------------------------- |
//! | Espressif            | ESP32       | ESP32-C6          | [`riot-rs-esp::*`](../../riot_rs_esp/index.html)     |
//! | Nordic Semiconductor | nRF         | nRF52840          | [`riot-rs-nrf::*`](../../riot_rs_nrf/index.html)     |
//! | Raspberry Pi         | RP          | RP2040            | [`riot-rs-rp::*`](../../riot_rs_rp/index.html)       |
//! | STMicroelectronics   | STM32       | STM32W55RGVX      | [`riot-rs-stm32::*`](../../riot_rs_stm32/index.html) |
//!
//! Documentation is only rendered for the MCUs listed in the table above, but [many others are
//! supported](https://future-proof-iot.github.io/RIOT-rs/dev/docs/book/hardware_functionality_support.html).
//! To render the docs locally for the MCU of your choice, adapt [the `cargo doc` command used to
//! generate documentation for the relevant
//! crate](https://github.com/future-proof-iot/RIOT-rs/blob/main/.github/workflows/build-deploy-docs.yml).

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
