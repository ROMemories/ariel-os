/// Registers a function acting as an entrypoint for an application.
///
/// The function is provided with peripherals, which can obtained by taking a peripheral struct
/// defined with `assign_peripherals!` as the first parameter.
///
/// # Parameters
///
/// - `usb_builder`: (*optional*) the macro will provide the function with a `UsbBuilderHook`.
///
/// # Panics
///
/// This macro panics when the `riot-rs` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};

    struct HookDefinition {
        type_name: &'static str,
        delegate_ident: &'static str,
        delegate_inner_type: proc_macro2::TokenStream,
        distributed_slice_type: proc_macro2::TokenStream,
    }

    let mut attrs = MainAttributes::default();
    let thread_parser = syn::meta::parser(|meta| attrs.parse(&meta));
    syn::parse_macro_input!(args with thread_parser);

    let task_function = syn::parse_macro_input!(item as syn::ItemFn);

    let riot_rs_crate = utils::riot_rs_crate();

    let hooks = &[HookDefinition {
        type_name: "UsbBuilderHook",
        delegate_ident: "USB_BUILDER_HOOK", // TODO: build this from type_name?
        delegate_inner_type: quote! { #riot_rs_crate::embassy::usb::UsbBuilder },
        distributed_slice_type: quote! { #riot_rs_crate::embassy::usb::USB_BUILDER_HOOKS },
    }];

    let delegate_type = quote! {#riot_rs_crate::embassy::delegate::Delegate};

    // Instantiate a Delegate as a static and store a reference to it in the appropriate
    // distributed slice
    let delegates = hooks.iter().map(|hook| {
        let HookDefinition { type_name, delegate_ident, delegate_inner_type, distributed_slice_type } = hook;

        let type_name = format_ident!("{type_name}");
        let delegate_hook_ident = format_ident!("{delegate_ident}");
        let delegate_hook_ref_ident = format_ident!("{delegate_ident}_REF");

        // TODO: try to reduce namespace pollution
        // FIXME: define the type in a way we can use it in the docs?
        quote! {
            static #delegate_hook_ident: #delegate_type<#delegate_inner_type> = #delegate_type::new();

            #[distributed_slice(#distributed_slice_type)]
            #[linkme(crate=#riot_rs_crate::embassy::linkme)]
            static #delegate_hook_ref_ident: &#delegate_type<#delegate_inner_type> = &#delegate_hook_ident;

            type #type_name = &'static #delegate_type<#delegate_inner_type>;
        }
    });

    let delegates = quote! { #(#delegates)* };

    let hook_args = hooks
        .iter()
        .map(|hook| format_ident!("{}_REF", hook.delegate_ident));
    let has_hook_args = hook_args.clone().count() > 0; // TODO: avoid this clone

    let hook_arg_list = if has_hook_args {
        quote! {, #(#hook_args),*}
    } else {
        quote! {}
    };

    let expanded = quote! {
        #delegates

        #[#riot_rs_crate::embassy::distributed_slice(#riot_rs_crate::embassy::EMBASSY_TASKS)]
        #[linkme(crate = #riot_rs_crate::embassy::linkme)]
        fn __main(
            spawner: &#riot_rs_crate::embassy::Spawner,
            mut peripherals: &mut #riot_rs_crate::embassy::arch::OptionalPeripherals,
        ) {
            use #riot_rs_crate::define_peripherals::IntoPeripherals;
            let task = main(peripherals.into_peripherals() #hook_arg_list);
            spawner.spawn(task).unwrap();
        }

        #[embassy_executor::task]
        #task_function
    };

    TokenStream::from(expanded)
}

#[derive(Default)]
struct MainAttributes {
    usb_builder: bool,
}

impl MainAttributes {
    // TODO: maybe enforce the order in which parameters are passed?
    fn parse(&mut self, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
        if meta.path.is_ident("usb_builder") {
            self.usb_builder = true;
            Ok(())
        } else {
            Err(meta.error("unsupported parameter"))
        }
    }
}
