#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::define_peripherals;

// As the macro will fail, this import will not get used
#[allow(unused_imports)]
use riot_rs::embassy::usb::UsbBuilderHook;

define_peripherals!(Peripherals {});

#[riot_rs::main]
async fn main(_peripherals: Peripherals, _usb_builder_hook: UsbBuilderHook) {}
