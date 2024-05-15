/// Label of a [`PhysicalValue`](crate::PhysicalValue) "dimension".
///
/// [`Label::Main`] is used for sensors returning a single [`PhysicalValue`]; even if a label
/// exists for the physical value the sensor measures, the [`Label::Main`] variant must be used.
/// Other labels are reserved for sensors measuring multiple physical values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, serde::Serialize)]
pub enum Label {
    Main,
    Humidity,
    Temperature,
    X,
    Y,
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
