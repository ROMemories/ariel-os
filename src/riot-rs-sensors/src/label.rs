/// Label of a [`PhysicalValue`](crate::sensor::PhysicalValue) part of a
/// [`PhysicalValues`](crate::sensor::PhysicalValues) set.
///
/// # For sensor driver implementors
///
/// [`Label::Main`] must be used for sensors returning a single
/// [`PhysicalValue`](crate::sensor::PhysicalValue), even if a more specific label exists for the
/// physical value.
/// This allows consumers displaying the label to ignore it for sensors measuring a single physical value.
/// Other labels are reserved for sensors measuring multiple physical values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize)]
pub enum Label {
    /// Used for sensors measuring a single physical value.
    Main,
    /// Humidity.
    Humidity,
    /// Temperature.
    Temperature,
    /// X axis.
    X,
    /// Y axis.
    Y,
    /// Z axis.
    Z,
}

// TODO: how to do i18n here?
impl core::fmt::Display for Label {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Main => write!(f, ""),
            Self::Humidity => write!(f, "Humidity"),
            Self::Temperature => write!(f, "Temperature"),
            Self::X => write!(f, "X"),
            Self::Y => write!(f, "Y"),
            Self::Z => write!(f, "Z"),
        }
    }
}
