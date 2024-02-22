/// Registers a function acting as an entrypoint for an application.
///
/// The function is provided with peripherals, which can obtained by taking a peripheral struct
/// defined with `assign_peripherals!` as the first parameter.
///
/// # Parameters
///
/// None.
///
/// # Panics
///
/// This macro panics when the `riot-rs` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    use quote::quote;

    let task_function = syn::parse_macro_input!(item as syn::ItemFn);

    let riot_rs_crate = utils::riot_rs_crate();

    let expanded = quote! {
        #[#riot_rs_crate::embassy::distributed_slice(#riot_rs_crate::embassy::EMBASSY_TASKS)]
        #[linkme(crate = #riot_rs_crate::embassy::linkme)]
        fn __main(spawner: &Spawner, mut peripherals: &mut #riot_rs_crate::embassy::arch::OptionalPeripherals) {
            use #riot_rs_crate::define_peripherals::IntoPeripherals;
            spawner.spawn(main(peripherals.into_peripherals())).unwrap();
        }

        #[embassy_executor::task]
        #task_function
    };

    TokenStream::from(expanded)
}
