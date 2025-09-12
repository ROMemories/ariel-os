use proc_macro::TokenStream;

#[proc_macro]
pub fn codegen_peripherals(_item: TokenStream) -> TokenStream {
    #[cfg(feature = "nrf")]
    let peripherals = embassy_nrf::PERIPHERALS;
    #[cfg(feature = "rp")]
    let peripherals = embassy_rp::PERIPHERALS;
    #[cfg(feature = "stm32")]
    let peripherals = embassy_stm32::PERIPHERALS;

    let peripherals = peripherals.into_iter().map(|p| quote::format_ident!("{p}"));

    quote::quote! {
        ariel_os_embassy_common::define_peripheral_struct!(
            input_peripherals = Peripherals,
            #(#peripherals),*
        );
    }
    .into()
}
