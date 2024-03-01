/// Registers a function acting as an entrypoint for an application.
///
/// The function is provided with peripherals, which can be obtained by taking a peripheral struct
/// defined with `assign_peripherals!` as the first parameter.
///
/// # Parameters
///
/// - `usb_builder`: (*optional*) when present, the macro will provide the function with a
/// `UsbBuilderHook`, allowing access and modification to the system-provided
/// `embassy_usb::Builder` through `Delegate::with()`, *before* it is built by the system.
///
/// # Examples
///
/// See RIOT-rs examples.
///
/// # Panics
///
/// This macro panics when the `riot-rs` crate cannot be found as a dependency of the crate where
/// this macro is used.
#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};

    #[allow(clippy::wildcard_imports)]
    use main_macro::*;

    let mut attrs = MainAttributes::default();
    let main_attr_parser = syn::meta::parser(|meta| attrs.parse(&meta));
    syn::parse_macro_input!(args with main_attr_parser);

    let task_function = syn::parse_macro_input!(item as syn::ItemFn);

    let riot_rs_crate = utils::riot_rs_crate();

    // New hooks need to be defined here, in the order they are run during system initialization
    let hooks = &[HookDefinition {
        kind: Hook::UsbBuilder,
        delegate_inner_type: quote! {#riot_rs_crate::embassy::usb::UsbBuilder},
        distributed_slice_type: quote! {#riot_rs_crate::embassy::usb::USB_BUILDER_HOOKS},
    }];

    let delegate_type = quote! {#riot_rs_crate::embassy::delegate::Delegate};

    let enabled_hooks = hooks.iter().filter(|hook| match hook.kind {
        Hook::UsbBuilder => attrs.usb_builder,
    });

    // Instantiate a Delegate as a static and store a reference to it in the appropriate
    // distributed slice
    let delegates = enabled_hooks.clone().map(|hook| {
        let HookDefinition { kind, delegate_inner_type, distributed_slice_type } = hook;

        let delegate_ident = kind.delegate_ident();

        let type_name = format_ident!("{}", kind.type_name());
        let delegate_hook_ident = format_ident!("{delegate_ident}");
        let delegate_hook_ref_ident = format_ident!("{delegate_ident}_REF");

        // TODO: try to reduce namespace pollution
        quote! {
            static #delegate_hook_ident: #delegate_type<#delegate_inner_type> = #delegate_type::new();

            #[#riot_rs_crate::embassy::distributed_slice(#distributed_slice_type)]
            #[linkme(crate=#riot_rs_crate::embassy::linkme)]
            static #delegate_hook_ref_ident: #type_name = &#delegate_hook_ident;
        }
    });

    let delegates = quote! {#(#delegates)*};

    let hook_args = enabled_hooks
        .clone()
        .map(|hook| format_ident!("{}_REF", hook.kind.delegate_ident()));
    let has_hook_args = hook_args.clone().count() > 0;

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

// Define these types in a module to avoid polluting the crate's namespace, as this file is
// `included!` in the crate's root.
mod main_macro {
    #[derive(Default)]
    pub struct MainAttributes {
        pub usb_builder: bool,
    }

    impl MainAttributes {
        // TODO: maybe enforce the order in which parameters are passed to this macro?
        pub fn parse(&mut self, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
            if meta.path.is_ident("usb_builder") {
                self.usb_builder = true;
                Ok(())
            } else {
                Err(meta.error("unsupported parameter"))
            }
        }
    }

    #[derive(Debug)]
    pub enum Hook {
        UsbBuilder,
    }

    impl Hook {
        pub fn type_name(&self) -> &'static str {
            match self {
                Self::UsbBuilder => "UsbBuilderHook",
            }
        }

        pub fn delegate_ident(&self) -> String {
            self.type_name().to_uppercase()
        }
    }

    #[derive(Debug)]
    pub struct HookDefinition {
        pub kind: Hook,
        pub delegate_inner_type: proc_macro2::TokenStream,
        pub distributed_slice_type: proc_macro2::TokenStream,
    }

}
