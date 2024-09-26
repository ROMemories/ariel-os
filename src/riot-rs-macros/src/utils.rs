use quote::format_ident;

pub const RIOT_RS_CRATE_NAME: &str = "riot-rs";

/// Returns a [`struct@syn::Ident`] identifying the `riot-rs` dependency.
///
/// # Panics
///
/// - Panics when the `riot-rs` crate cannot be found as a dependency of the crate in which
///   this function is called.
/// - Panics if `riot-rs` is used as a dependency of itself.
pub fn riot_rs_crate() -> Option<syn::Ident> {
    find_crate(RIOT_RS_CRATE_NAME)
}

/// Returns a [`struct@syn::Ident`] identifying the `name` dependency (or `None`).
///
/// # Panics
///
/// - Panics if `name` is used as a dependency of itself.
pub fn find_crate(name: &str) -> Option<syn::Ident> {
    if let Ok(crate_) = proc_macro_crate::crate_name(name) {
        match crate_ {
            proc_macro_crate::FoundCrate::Itself => {
                panic!("{name} cannot be used as a dependency of itself");
            }
            proc_macro_crate::FoundCrate::Name(crate_) => Some(format_ident!("{crate_}")),
        }
    } else {
        None
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

pub fn parse_parent_module_path(path: &str) -> String {
    // FIXME: is this robust enough? could we parse with syn first?
    path.split("::<")
        .next()
        .unwrap()
        .rsplit("::")
        .skip(1)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("::")
}

pub fn parse_type_name_from_type_path(path: &str) -> &str {
    // FIXME: is this robust enough? could we parse with syn first?
    path.split("::<")
        .next()
        .unwrap()
        .rsplit("::")
        .take(1)
        .next()
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_parent_module_path() {
        assert_eq!(
            parse_parent_module_path(
                "riot_rs::builtin_sensors::lis3dh::Lis3dh::<riot_rs::emb↴
                                                 …assy::arch::i2c::I2c>",
            ),
            "riot_rs::builtin_sensors::lis3dh",
        );
    }

    #[test]
    fn test_parse_type_name_from_type_path() {
        assert_eq!(
            parse_type_name_from_type_path(
                "riot_rs::builtin_sensors::lis3dh::Lis3dh::<riot_rs::emb↴
                                                 …assy::arch::i2c::I2c>",
            ),
            "Lis3dh",
        );
    }
}
