#[proc_macro]
pub fn define_count_adjusted_enums(_item: TokenStream) -> TokenStream {
    use quote::quote;

    #[allow(clippy::wildcard_imports)]
    use define_count_adjusted_enum::*;

    // The order of these feature-gated statements is important as these features are not meant to
    // be mutually exclusive.
    #[expect(unused_variables, reason = "overridden by feature selection")]
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

    let reading_infos_variants = (1..=count).map(|i| {
        let variant = variant_name(i);
        quote! { #variant([ReadingInfo; #i]) }
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
        /// This type is automatically generated, the number of variants is automatically adjusted.
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

        /// Meta-data required to interpret values returned by [`Sensor::read()`].
        ///
        /// The order matches the one of [`PhysicalValues`].
        ///
        /// # Note
        ///
        /// This type is automatically generated, the number of variants is automatically adjusted.
        #[derive(Debug, Copy, Clone)]
        pub enum ReadingInfos {
            #(#reading_infos_variants),*
        }

        impl ReadingInfos {
            pub fn iter(&self) -> impl Iterator<Item = ReadingInfo> + '_ {
                match self {
                    #(#values_iter),*
                }
            }

            pub fn first(&self) -> ReadingInfo {
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
