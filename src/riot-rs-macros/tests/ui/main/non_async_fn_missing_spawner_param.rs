#![no_main]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

// FAIL: non-async function is expected to take a Spawner as first parameter
#[riot_rs::main]
fn main(foo: Bar) {}

struct Bar;
