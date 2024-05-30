//! See your architecture's Embassy crate documentation.

use core::{convert::Infallible, marker::PhantomData};

pub struct Input<'d> {
    pin: Flex<'d>,
}

pub struct Flex<'d> {
    pin: PhantomData<&'d ()>,
}

pub struct Output<'d> {
    pin: Flex<'d>,
}

impl<'d> embedded_hal::digital::OutputPin for Output<'d> {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        unimplemented!();
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        unimplemented!();
    }
}

impl<'d> embedded_hal::digital::ErrorType for Output<'d> {
    type Error = Infallible;
}
