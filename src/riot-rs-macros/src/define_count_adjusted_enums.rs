#[proc_macro]
pub fn define_count_adjusted_enums(_item: TokenStream) -> TokenStream {
    use quote::quote;

    #[allow(clippy::wildcard_imports)]
    use define_count_adjusted_enum::*;

    // The order of these feature-gated statements is important as these features are not meant to
    // be mutually exclusive.
    let count = 1;
    #[cfg(feature = "max-reading-value-min-count-2")]
    let count = 2;
    #[cfg(feature = "max-reading-value-min-count-3")]
    let count = 3;
    #[cfg(feature = "max-reading-value-min-count-9")]
    let count = 9;
    #[cfg(feature = "max-reading-value-min-count-12")]
    let count = 12;

    let physical_values_variants = (1..=count).map(|i| {
        let variant = variant_name(i);
        quote! { #variant([PhysicalValue; #i]) }
    });
    let physical_values_first_value = (1..=count).map(|i| {
        let variant = variant_name(i);
        // TODO: can we do this without an unwrap?
        quote! { Self::#variant(values) => *values.first().unwrap() }
    });

    let value_scales_variants = (1..=count).map(|i| {
        let variant = variant_name(i);
        quote! { #variant([i8; #i]) }
    });

    let physical_units_variants = (1..=count).map(|i| {
        let variant = variant_name(i);
        quote! { #variant([PhysicalUnit; #i]) }
    });

    let labels_variants = (1..=count).map(|i| {
        let variant = variant_name(i);
        quote! { #variant([Label; #i]) }
    });

    let values_iter = (1..=count)
        .map(|i| {
            let variant = variant_name(i);
            quote! { Self::#variant(values) => values.iter().copied() }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        // TODO: add a timestamp?
        /// Values returned by a sensor.
        ///
        /// The order of [`PhysicalValue`]s is not significant, but is fixed.
        ///
        /// # Note
        ///
        /// This type is automatically generated, the number of variants is controlled by
        /// `riot_rs_hwsetup::Sensors::max_reading_value_count()`.
        #[derive(Debug, Copy, Clone, serde::Serialize)]
        pub enum PhysicalValues {
            #(#physical_values_variants),*
        }

        impl Reading for PhysicalValues {
            fn value(&self) -> PhysicalValue {
                match self {
                    #(#physical_values_first_value),*
                }
            }

            fn values(&self) -> impl ExactSizeIterator<Item = PhysicalValue> {
                match self {
                    #(#values_iter),*
                }
            }
        }

        /// Scaling values of [`PhysicalValues`] returned by [`Sensor::read()`].
        ///
        /// The order matches the one of [`PhysicalValues`].
        ///
        /// # Note
        ///
        /// This type is automatically generated, the number of variants is controlled by
        /// `riot_rs_hwsetup::Sensors::max_reading_value_count()`.
        #[derive(Debug, Copy, Clone)]
        pub enum ValueScales {
            #(#value_scales_variants),*
        }

        impl ValueScales {
            pub fn iter(&self) -> impl Iterator<Item = i8> + '_ {
                match self {
                    #(#values_iter),*
                }
            }

            pub fn first(&self) -> i8 {
                // NOTE(no-panic): there is always at least one value.
                self.iter().next().unwrap()
            }
        }

        /// Units of measurement of [`PhysicalValues`] returned by [`Sensor::read()`].
        ///
        /// The order matches the one of [`PhysicalValues`].
        ///
        /// # Note
        ///
        /// This type is automatically generated, the number of variants is controlled by
        /// `riot_rs_hwsetup::Sensors::max_reading_value_count()`.
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum PhysicalUnits {
            #(#physical_units_variants),*
        }

        impl PhysicalUnits {
            pub fn iter(&self) -> impl Iterator<Item = PhysicalUnit> + '_ {
                match self {
                    #(#values_iter),*
                }
            }

            pub fn first(&self) -> PhysicalUnit {
                // NOTE(no-panic): there is always at least one value.
                self.iter().next().unwrap()
            }
        }

        /// [`Label`]s of [`PhysicalValues`] returned by [`Sensor::read()`].
        ///
        /// The order matches the one of [`PhysicalValues`].
        ///
        /// # Note
        ///
        /// This type is automatically generated, the number of variants is controlled by
        /// `riot_rs_hwsetup::Sensors::max_reading_value_count()`.
        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub enum Labels {
            #(#labels_variants),*
        }

        impl Labels {
            pub fn iter(&self) -> impl Iterator<Item = Label> + '_ {
                match self {
                    #(#values_iter),*
                }
            }

            pub fn first(&self) -> Label {
                // NOTE(no-panic): there is always at least one value.
                self.iter().next().unwrap()
            }
        }
    };

    TokenStream::from(expanded)
}

mod define_count_adjusted_enum {
    pub fn variant_name(index: usize) -> syn::Ident {
        quote::format_ident!("V{index}")
    }
}
