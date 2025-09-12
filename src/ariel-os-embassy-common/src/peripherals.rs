//! Shared items related to peripherals.

/// Define the custom peripheral struct.
///
/// The list of peripherals can be codegened by obtaining it from the HAL crate if it exposes it in
/// some way.
/// Alternatively, the following command can be used to obtain the list of Embassy peripherals for
/// a given MCU, by scraping the documentation generated with `cargo doc`:
///
/// ```sh
/// embassy_crate='embassy_rp'; \
///     grep --only-matching 'title="struct embassy_[^:]\+::peripherals::[^"]\+' "target/doc/${embassy_crate}/peripherals/index.html" \
///     | sed 's/title="struct embassy_[^:]\+::peripherals:://'
/// ```
///
/// The documentation page can also be obtained from the published documentation with the
/// following:
///
/// ```sh
/// embassy_crate='embassy-nrf'; chip='nrf52840'; \
///   wget -q --output-document - "https://docs.embassy.dev/${embassy_crate}/git/${chip}/peripherals/index.html"
/// ```
#[macro_export]
macro_rules! define_peripheral_struct {
    (
        input_peripherals = $input_peripherals:path,
        $( $peripheral:tt ),* $(,)?
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
