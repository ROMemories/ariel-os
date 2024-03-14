/// Registers a non-async function acting as an entrypoint for an application.
///
/// The function is provided with:
///
/// - a `Spawner` as first parameter,
/// - a peripheral struct, as optional second parameter.
///
/// The peripheral struct must be defined with the `define_peripherals!` macro.
///
/// See [`macro@main`] to use a long-lived async function instead.
///
/// # Parameters
///
/// None.
///
/// # Examples
///
/// ```ignore
/// use riot_rs::embassy::Spawner;
///
/// #[riot_rs::spawner]
/// fn spawner(spawner: Spawner, peripherals: /* your peripheral type */) {}
/// ```
///
/// See RIOT-rs examples for more.
///
/// # Panics
///
/// This macro panics when the `riot-rs` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro_attribute]
pub fn spawner(_args: TokenStream, item: TokenStream) -> TokenStream {
    use quote::quote;

    let main_function = syn::parse_macro_input!(item as syn::ItemFn);
    let main_function_name = &main_function.sig.ident;
    let is_async = main_function.sig.asyncness.is_some();

    assert!(!is_async, "spawner functions cannot be async");

    let riot_rs_crate = utils::riot_rs_crate();

    let takes_peripherals = !main_function.sig.inputs.len() > 1;
    let peripheral_param = if takes_peripherals {
        quote! {, peripherals.take_peripherals()}
    } else {
        quote! {}
    };

    let expanded = quote! {
        #[#riot_rs_crate::embassy::distributed_slice(#riot_rs_crate::embassy::EMBASSY_TASKS)]
        #[linkme(crate = #riot_rs_crate::embassy::linkme)]
        fn __main(
            spawner: #riot_rs_crate::embassy::Spawner,
            mut peripherals: &mut #riot_rs_crate::embassy::arch::OptionalPeripherals,
        ) {
            use #riot_rs_crate::define_peripherals::TakePeripherals;
            #main_function_name(spawner #peripheral_param);
        }

        #main_function
    };

    TokenStream::from(expanded)
}
