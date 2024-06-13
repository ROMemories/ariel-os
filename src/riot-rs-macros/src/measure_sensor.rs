/// Calls `Sensor::measure()` on a sensor trait object.
///
/// # Panics
///
/// This macro panics when the `riot-rs` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro]
pub fn measure_sensor(input: TokenStream) -> TokenStream {
    use quote::quote;
    use riot_rs_hwsetup::{
        sensors::{Sensor, StringOrTypePath},
        HwSetup,
    };

    let params = syn::parse_macro_input!(input as await_sensor::Params);
    let sensor_ident = params.sensor_ident;

    let hwsetup_path = HwSetup::get_path_from_env().unwrap();
    let hwsetup = HwSetup::read_from_path(&hwsetup_path).unwrap();
    // dbg!(&hwsetup);

    let sensor_type_list = hwsetup.sensors().connected().iter().map(|sensor_setup| {
        match StringOrTypePath::from(sensor_setup.driver()) {
            StringOrTypePath::TypePath(type_path) => {
                let cfg_conds = utils::parse_cfg_conditionals(sensor_setup);
                let type_path = utils::parse_type_path(type_path);

                quote! {
                    #[cfg(all(#(#cfg_conds),*))]
                    #type_path
                }
            },
            _ => panic!("`driver` must start with an @"),
        }
    });

    let sensors_mod = if let Some(riot_rs_crate) = utils::riot_rs_crate() {
        quote! { #riot_rs_crate::sensors }
    }else {
        quote! { riot_rs_sensors }
    };

    // FIXME: we should generate the macro used by users in this macro, instead of doing the
    // opposite, so that the hw config file only gets parsed once

    // The `_measure_sensor` macro expects a trailing comma
    let expanded = quote! {
        #sensors_mod::_measure_sensor!(#sensor_ident, #(#sensor_type_list),* ,)
    };

    TokenStream::from(expanded)
}

mod await_sensor {
    use syn::{
        parse::{Parse, ParseStream},
        ExprPath,
    };

    #[derive(Debug)]
    pub struct Params {
        pub sensor_ident: ExprPath,
    }

    impl Parse for Params {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let sensor_ident = input.parse()?;

            Ok(Self { sensor_ident })
        }
    }
}
