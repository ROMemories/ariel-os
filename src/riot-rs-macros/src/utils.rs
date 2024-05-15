use quote::format_ident;

const RIOT_RS_CRATE_NAME: &str = "riot-rs";

/// Returns a [`struct@syn::Ident`] identifying the `riot-rs` dependency.
///
/// # Panics
///
/// - Panics when the `riot-rs` crate cannot be found as a dependency of the crate in which
/// this function is called.
/// - Panics if `riot-rs` is used as a dependency of itself.
pub fn riot_rs_crate() -> syn::Ident {
    let riot_rs_crate = proc_macro_crate::crate_name(RIOT_RS_CRATE_NAME)
        .unwrap_or_else(|_| panic!("{RIOT_RS_CRATE_NAME} should be present in `Cargo.toml`"));

    match riot_rs_crate {
        proc_macro_crate::FoundCrate::Itself => {
            panic!(
                "{} cannot be used as a dependency of itself",
                env!("CARGO_CRATE_NAME"),
            );
        }
        proc_macro_crate::FoundCrate::Name(riot_rs_crate) => format_ident!("{riot_rs_crate}"),
    }
}

pub fn parse_type_path(type_path: &str) -> proc_macro2::TokenStream {
    // TODO: is this the right type of path?
    let path = syn::parse_str::<syn::TypePath>(type_path).unwrap();
    quote::quote! {#path}
}

pub fn bool_as_token(boolean: bool) -> proc_macro2::TokenStream {
    if boolean {
        quote::quote! { true }
    } else {
        quote::quote! { false }
    }
}

pub fn parse_cfg_conditionals(
    conditioned_setup: &impl riot_rs_hwsetup::Conditioned,
) -> Vec<proc_macro2::TokenStream> {
    let on_conds = parse_conditional_list("context", conditioned_setup.on());
    let when_conds = parse_conditional_list("feature", conditioned_setup.when());

    on_conds
        .into_iter()
        .chain(when_conds.into_iter())
        .collect::<Vec<_>>()
}

fn parse_conditional_list(
    cfg_attr: &str,
    conditionals: Option<&str>,
) -> Vec<proc_macro2::TokenStream> {
    if let Some(on) = conditionals {
        let context_attr = format_ident!("{cfg_attr}");

        on.split(',')
            .map(str::trim)
            .map(|context| quote::quote!(#context_attr = #context))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    }
}
