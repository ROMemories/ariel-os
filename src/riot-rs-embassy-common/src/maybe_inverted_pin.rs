//! Provides a possibly inverted pin.

// Based on
// https://github.com/eldruin/inverted-pin-rs/blob/e392a0910bd8a180ceef55d82df3cfbedeac5921/src/inverted.rs
// under MIT license

use embedded_hal::digital::{Error, ErrorType, InputPin};

/// Possibly inverted input pin.
///
/// Whether the pin is inverted is configurable at instantiation.
// TODO: try to make this more memory-efficient, and evaluate runtime overhead.
#[derive(Debug, Clone, Copy)]
pub struct MaybeInvertedPin<P> {
    pin: P,
    inverted: bool,
}

impl<P> MaybeInvertedPin<P> {
    /// Returns a new possibly inverted pin.
    /// If `inverted` is `false`, the pin will not be inverted.
    pub fn new(pin: P, inverted: bool) -> Self {
        Self { pin, inverted }
    }

    /// Destroys the instance and returns the wrapped pin.
    pub fn destroy(self) -> P {
        self.pin
    }
}

impl<P, E> ErrorType for MaybeInvertedPin<P>
where
    P: ErrorType<Error = E>,
    E: Error,
{
    type Error = E;
}

impl<P, E> InputPin for MaybeInvertedPin<P>
where
    P: InputPin<Error = E>,
    E: Error,
{
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        Ok(!(self.inverted ^ self.pin.is_low()?))
    }

    fn is_low(&mut self) -> Result<bool, Self::Error> {
        Ok(!(self.inverted ^ self.pin.is_high()?))
    }
}
