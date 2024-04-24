//! See your architecture's Embassy crate documentation.

use core::marker::PhantomData;

pub struct Input<'d> {
    pin: Flex<'d>,
}

pub struct Flex<'d> {
    pin: PhantomData<&'d ()>,
}

struct AnyPin {}
