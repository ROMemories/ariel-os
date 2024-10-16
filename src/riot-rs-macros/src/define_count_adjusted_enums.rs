#[proc_macro]
pub fn define_count_adjusted_enums(_item: TokenStream) -> TokenStream {
    use quote::quote;

    #[allow(clippy::wildcard_imports)]
    use define_count_adjusted_enum::*;

    // The order of these feature-gated statements is important as these features are not meant to
    // be mutually exclusive.
    #[allow(unused_variables, reason = "overridden by feature selection")]
    let count = 1;
    #[cfg(feature = "max-reading-value-min-count-2")]
    let count = 2;
    #[cfg(feature = "max-reading-value-min-count-3")]
    let count = 3;
    #[cfg(feature = "max-reading-value-min-count-6")]
    let count = 6;
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

    let reading_axes_variants = (1..=count).map(|i| {
        let variant = variant_name(i);
        quote! { #variant([ReadingAxis; #i]) }
    });

    let values_iter = (1..=count)
        .map(|i| {
            let variant = variant_name(i);
            quote! { Self::#variant(values) => values.iter().copied() }
        })
        .collect::<Vec<_>>();

    let expanded = quote! {
        // TODO: add a timestamp?
        /// Values returned by a sensor driver.
        ///
        /// This type implements [`Reading`] to iterate over the values.
        ///
        /// # Note
        ///
        /// This type is automatically generated, the number of variants is automatically adjusted.
        #[derive(Debug, Copy, Clone, serde::Serialize)]
        pub enum PhysicalValues {
            #[doc(hidden)]
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

        /// Metadata required to interpret values returned by [`Sensor::wait_for_reading()`].
        ///
        /// # Note
        ///
        /// This type is automatically generated, the number of variants is automatically adjusted.
        #[derive(Debug, Copy, Clone)]
        pub enum ReadingAxes {
            #[doc(hidden)]
            #(#reading_axes_variants),*,
        }

        impl ReadingAxes {
            /// Returns an iterator over the underlying [`ReadingAxis`] items.
            ///
            /// For a given sensor driver, the number and order of items match the one of
            /// [`PhysicalValues`].
            /// [`Iterator::zip()`] can be useful to zip the returned iterator with the one
            /// obtained with [`Reading::values()`].
            pub fn iter(&self) -> impl Iterator<Item = ReadingAxis> + '_ {
                match self {
                    #(#values_iter),*,
                }
            }

            /// Returns the first [`ReadingAxis`].
            pub fn first(&self) -> ReadingAxis {
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
