/// TODO
///
/// # Parameters
///
/// TODO
///
/// # Examples
///
/// TODO
///
/// # Panics
///
/// This macro panics when the `riot-rs` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    use std::iter;

    use quote::quote;

    let task_function = syn::parse_macro_input!(item as syn::ItemFn);
    let function_param_count = task_function.sig.inputs.len();

    // Add an extractor for each function parameter, so that the function receives the requested
    // peripherals
    let extractors =
        iter::repeat(quote! { peripherals.into_peripherals() }).take(function_param_count);

    let riot_rs_crate = utils::riot_rs_crate();

    let expanded = quote! {
        #[#riot_rs_crate::embassy::distributed_slice(#riot_rs_crate::embassy::EMBASSY_TASKS)]
        #[linkme(crate = #riot_rs_crate::embassy::linkme)]
        fn __main(spawner: &Spawner, mut peripherals: &mut #riot_rs_crate::embassy::arch::OptionalPeripherals) {
            use #riot_rs_crate::define_peripherals::FromOptionalPeripherals;
            spawner.spawn(main(#(#extractors),*)).unwrap();
        }

        #[embassy_executor::task]
        #task_function
    };

    TokenStream::from(expanded)
}
