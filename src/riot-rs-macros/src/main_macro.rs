/// Registers a function acting as an entrypoint for an application.
///
/// The function is provided with peripherals, which can be obtained by taking a peripheral struct
/// defined with `assign_peripherals!` as the first parameter.
///
/// # Parameters
///
/// - `peripherals`: (*optional*) list of types defined using the `define_peripherals!` macro that
/// will be provided to the function, in order.
/// - `hooks`: (*optional*) list of hooks. Available hooks are:
///     - `usb_builder`: (*optional*) when present, the macro will provide the function with a
///     `UsbBuilderHook`, allowing access and modification to the system-provided
///     `embassy_usb::Builder` through `Delegate::with()`, *before* it is built by the system.
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
    use quote::quote;

    #[allow(clippy::wildcard_imports)]
    use main_macro::*;

    let mut attrs = MainAttributes::default();
    let main_attr_parser = syn::meta::parser(|meta| attrs.parse(&meta));
    syn::parse_macro_input!(args with main_attr_parser);

    let main_function = syn::parse_macro_input!(item as syn::ItemFn);
    let main_function_name = &main_function.sig.ident;
    let is_async = main_function.sig.asyncness.is_some();

    let riot_rs_crate = utils::riot_rs_crate();

    // New hooks need to be defined here, in the order they are run during system initialization
    let hooks = &[HookDefinition {
        kind: Hook::UsbBuilder,
        delegate_inner_type: quote! {#riot_rs_crate::embassy::usb::UsbBuilder},
        distributed_slice_type: quote! {#riot_rs_crate::embassy::usb::USB_BUILDER_HOOKS},
    }];

    let expanded = if is_async {
        let (delegates, hook_arg_list) =
            main_macro::generate_delegates(&riot_rs_crate, hooks, &attrs);

        quote! {
            #delegates

            #[#riot_rs_crate::embassy::distributed_slice(#riot_rs_crate::embassy::EMBASSY_TASKS)]
            #[linkme(crate = #riot_rs_crate::embassy::linkme)]
            fn __main(
                spawner: #riot_rs_crate::embassy::Spawner,
                mut peripherals: &mut #riot_rs_crate::embassy::arch::OptionalPeripherals,
            ) {
                use #riot_rs_crate::define_peripherals::TakePeripherals;
                let task = #main_function_name(peripherals.take_peripherals() #hook_arg_list);
                spawner.spawn(task).unwrap();
            }

            #[#riot_rs_crate::embassy_executor::task]
            #main_function
        }
    } else {
        quote! {
            #[#riot_rs_crate::embassy::distributed_slice(#riot_rs_crate::embassy::EMBASSY_TASKS)]
            #[linkme(crate = #riot_rs_crate::embassy::linkme)]
            fn __main(
                spawner: #riot_rs_crate::embassy::Spawner,
                mut peripherals: &mut #riot_rs_crate::embassy::arch::OptionalPeripherals,
            ) {
                use #riot_rs_crate::define_peripherals::TakePeripherals;
                #main_function_name(spawner, peripherals.take_peripherals());
            }

            #main_function
        }
    };

    TokenStream::from(expanded)
}

// Define these types in a module to avoid polluting the crate's namespace, as this file is
// `included!` in the crate's root.
mod main_macro {
    #[derive(Debug, Default)]
    pub struct MainAttributes {
        pub peripheral_types: Vec<syn::Path>,
        pub hooks: Vec<Hook>,
    }

    impl MainAttributes {
        // TODO: maybe enforce the order in which parameters are passed to this macro?
        pub fn parse(&mut self, attr: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
            if attr.path.is_ident("hooks") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident(Hook::UsbBuilder.param_name()) {
                        self.hooks.push(Hook::UsbBuilder);
                        Ok(())
                    } else {
                        let supported_hooks = Hook::format_list();
                        Err(meta.error(format!(
                            "unsupported hook ({supported_hooks} are supported)"
                        )))
                    }
                })
            } else {
                Err(attr.error("unsupported parameter (`hooks` is supported)"))
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, Hash, enum_iterator::Sequence)]
    pub enum Hook {
        UsbBuilder,
    }

    impl Hook {
        pub fn param_name(&self) -> &'static str {
            match self {
                Self::UsbBuilder => "usb_builder",
            }
        }

        pub fn type_name(&self) -> &'static str {
            match self {
                Self::UsbBuilder => "UsbBuilderHook",
            }
        }

        pub fn delegate_ident(&self) -> String {
            self.type_name().to_uppercase()
        }

        fn format_list() -> String {
            enum_iterator::all::<Self>()
                .map(|h| format!("`{}`", h.param_name()))
                .collect::<Vec<_>>()
                .join(", ")
        }
    }

    #[derive(Debug)]
    pub struct HookDefinition {
        pub kind: Hook,
        pub delegate_inner_type: proc_macro2::TokenStream,
        pub distributed_slice_type: proc_macro2::TokenStream,
    }

    pub fn generate_delegates(
        riot_rs_crate: &syn::Ident,
        hooks: &[HookDefinition],
        attrs: &MainAttributes,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        use quote::{format_ident, quote};

        let delegate_type = quote! {#riot_rs_crate::embassy::delegate::Delegate};

        let enabled_hooks = hooks.iter().filter(|hook| match hook.kind {
            Hook::UsbBuilder => attrs.hooks.iter().any(|h| *h == Hook::UsbBuilder),
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
            }
        );

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

        (delegates, hook_arg_list)
    }
}
