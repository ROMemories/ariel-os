#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

// As the macro will fail, this import will not get used
#[allow(unused_imports)]
use riot_rs::embassy::usb::UsbBuilderHook;

// FAIL: the function is expected to take a type having a `take_peripherals()` method as first
// parameter
#[riot_rs::main]
async fn main(_foo: Bar) {}

struct Bar;
