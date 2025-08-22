//! Shared items related to peripherals.

/// Define the custom peripheral struct.
///
/// The following command can be used to obtain the list of Embassy peripherals for a given MCU, by
/// scraping the published documentation:
///
/// ```sh
/// embassy_crate='embassy-nrf'; chip='nrf52840'; \
///   wget -q --output-document - "https://docs.embassy.dev/${embassy_crate}/git/${chip}/peripherals/index.html" \
///   | grep --only-matching 'title="struct embassy_[^:]\+::peripherals::[^"]\+' \
///   | sed 's/title="struct embassy_[^:]\+::peripherals:://'
/// ```
#[macro_export]
macro_rules! define_peripheral_struct {
    (
        input_peripherals = $input_peripherals:path,
        $( $peripheral:ident ),* $(,)?
    ) => {
        /// Peripheral struct that allows partial moves.
        #[allow(non_snake_case)]
        #[allow(dead_code)]
        #[allow(missing_docs)]
        pub struct OptionalPeripherals {
            $( pub $peripheral: Option<peripherals::$peripheral> ),*
        }

        impl OptionalPeripherals {
            pub(crate) const fn from(p: $input_peripherals) -> Self {
                Self {
                    $( $peripheral: Some(p.$peripheral) ),*
                }
            }
        }
    }
}
