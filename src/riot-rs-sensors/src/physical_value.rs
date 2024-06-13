/// Represents a value obtained from a sensor.
///
/// The [scaling value](crate::sensor::ReadingInfo::scaling()) obtained from the sensor with
/// [`Sensor::reading_infos()`](crate::Sensor::reading_infos) must be taken into account using the
/// following formula:
///
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mrow><mi mathvariant="monospace">PhysicalValue::value()</mi></mrow><mo>·</mo><msup><mn>10</mn><mrow><mi mathvariant="monospace">scaling</mi></mrow></msup></math>
///
/// For instance, in the case of a temperature sensor, if [`PhysicalValue::value()`] returns `2225`
/// and the scaling value is `-2`, this means that the temperature measured and returned by the
/// hardware sensor is `22.25` (the [measurement error](PhysicalValue::error()) must additionally
/// be taken into account).
///
/// The unit of measurement can be obtained using
/// [`ReadingInfo::unit()`](crate::sensor::ReadingInfo::unit).
///
/// This is required to avoid handling floats.
#[derive(Debug, Copy, Clone, serde::Serialize)]
pub struct PhysicalValue {
    value: i32,
    error: AccuracyError,
}

impl PhysicalValue {
    /// Creates a new value.
    #[must_use]
    pub const fn new(value: i32, error: AccuracyError) -> Self {
        Self { value, error }
    }

    /// Returns the value.
    #[must_use]
    pub fn value(&self) -> i32 {
        self.value
    }

    /// Returns the measurement error.
    #[must_use]
    pub fn error(&self) -> AccuracyError {
        self.error
    }
}

/// Specifies the accuracy error of a measurement.
///
/// It is assumed that the accuracy error is symmetrical around a possibly non-zero bias.
///
/// The unit of measurement is provided by the [`ReadingInfo`](crate::sensor::ReadingInfo)
/// associated to the [`PhysicalValue`].
/// The `scaling` value is used for both `deviation` and `bias`.
/// The accuracy error is thus given by the following formulas:
///
/// <math xmlns="http://www.w3.org/1998/Math/MathML" display="block"><mo>+</mo><mo>(</mo><mrow><mi mathvariant="monospace">bias</mi></mrow><mo>+</mo><mrow><mi mathvariant="monospace">deviation</mi></mrow><mo>)</mo><mo>·</mo><msup><mn>10</mn><mrow><mi mathvariant="monospace">scaling</mi></mrow></msup>/<mo>-</mo><mo>(</mo><mrow><mi mathvariant="monospace">bias</mi></mrow><mo>-</mo><mrow><mi mathvariant="monospace">deviation</mi></mrow><mo>)</mo><mo>·</mo><msup><mn>10</mn><mrow><mi mathvariant="monospace">scaling</mi></mrow></msup></math>
///
/// # Examples
///
/// The DS18B20 temperature sensor accuracy error is <mo>+</mo><mn>0.05</mn>/<mo>-</mo><mn>0.45</mn>
/// at 20 °C (see Figure 1 of its datasheet).
/// [`AccuracyError`] would thus be the following:
///
/// ```
/// # use riot_rs_sensors::sensor::AccuracyError;
/// AccuracyError {
///     deviation: 25,
///     bias: -20,
///     scaling: -2,
/// }
/// # ;
/// ```
#[derive(Debug, Copy, Clone, serde::Serialize)]
pub enum AccuracyError {
    /// Unknown measurement error.
    Unknown,
    /// No measurement error (e.g., boolean values).
    None,
    /// Measurement error symmetrical around the [`bias`](AccuracyError::Symmetrical::bias).
    Symmetrical {
        /// Deviation around the bias value.
        deviation: i16, // FIXME: rename this?
        /// Bias (mean accuracy error).
        bias: i16,
        /// Scaling of [`deviation`](AccuracyError::Symmetrical::deviation) and
        /// [`bias`](AccuracyError::Symmetrical::bias).
        scaling: i8,
    },
}

/// Implemented on types returned by [`Sensor::measure()`](crate::Sensor::measure).
///
/// [`PhysicalValues`](crate::sensor::PhysicalValues) implements this trait, and should usually be
/// used by sensor driver implementors.
pub trait Reading: core::fmt::Debug {
    /// Returns the [`PhysicalValue`] of a sensor reading.
    ///
    /// This returns the first value returned by [`Reading::values()`].
    fn value(&self) -> PhysicalValue;

    /// Returns an iterator over [`PhysicalValue`]s of a sensor reading.
    /// The order of [`PhysicalValue`]s is not significant, but is fixed.
    ///
    /// # For implementors
    ///
    /// The default implementation must be overridden on types containing multiple
    /// [`PhysicalValue`]s.
    fn values(&self) -> impl ExactSizeIterator<Item = PhysicalValue> {
        [self.value()].into_iter()
    }
}
