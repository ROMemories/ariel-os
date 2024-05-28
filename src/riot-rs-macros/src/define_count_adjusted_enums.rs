#[proc_macro]
pub fn define_count_adjusted_enums(_item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::HwSetup;

    // Allow tooling to run without passing a setup file
    let hwsetup = if let Ok(hwsetup_path) = HwSetup::get_path_from_env() {
        HwSetup::read_from_path(&hwsetup_path).unwrap()
    } else {
        HwSetup::default()
    };

    fn variant_name(index: usize) -> syn::Ident {
        format_ident!("V{index}")
    }

    let count = usize::from(u8::from(hwsetup.sensors().max_reading_value_count()));
    dbg!(&count);

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
        }

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
        }

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
        }
    };

    TokenStream::from(expanded)
}

mod define_count_adjusted_enum {}
