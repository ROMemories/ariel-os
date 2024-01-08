use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn run(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::DeriveInput);

    let struct_name = input.ident.clone();

    let expanded = quote! {
        #input

        riot_rs::embassy::riot_initialize!(#struct_name);
    };

    TokenStream::from(expanded)
}
