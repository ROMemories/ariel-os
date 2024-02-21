/// Registers the function this attribute macro is applied on to provide the configuration for the
/// associated driver during initial system configuration.
///
/// The name of the function does not matter as it will be renamed by the macro.
///
/// # Parameters
///
/// - The name of the driver the function provides configuration for.
///
/// | Driver    | Expected return type           |
/// | --------- | ------------------------------ |
/// | `network` | `embassy_net::Config`          |
/// | `usb`     | `embassy_usb::Config<'static>` |
///
/// # Note
///
/// The `riot_rs` crate provides re-exports for the relevant Embassy crates.
///
/// # Examples
///
/// The following function provides configuration for the network stack:
///
/// ```ignore
/// use riot_rs::embassy_net;
///
/// #[riot_rs::config(network)]
/// fn network_config() -> embassy_net::Config {
///     use embassy_net::Ipv4Address;
///
///     embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
///         address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
///         dns_servers: heapless::Vec::new(),
///         gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
///     })
/// }
/// ```
#[allow(clippy::missing_panics_doc)]
#[proc_macro_attribute]
pub fn config(args: TokenStream, item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};

    let mut attrs = ConfigAttributes::default();
    let thread_parser = syn::meta::parser(|meta| attrs.parse(&meta));
    syn::parse_macro_input!(args with thread_parser);

    let config_function = syn::parse_macro_input!(item as syn::ItemFn);
    let config_function_name = &config_function.sig.ident;

    let riot_rs_crate = utils::riot_rs_crate();

    let (config_fn_name, return_type) = match attrs.kind {
        Some(ConfigKind::Network) => (
            format_ident!("riot_rs_network_config"),
            quote! {#riot_rs_crate::embassy::embassy_net::Config},
        ),
        Some(ConfigKind::Usb) => (
            format_ident!("riot_rs_usb_config"),
            quote! {#riot_rs_crate::embassy::embassy_usb::Config<'static>},
        ),
        None => {
            panic!("a configuration kind must be specified");
        }
    };

    // Place the provided function into another function whose type signature we enforce.
    // This is important as that function will be called unsafely via FFI.
    let expanded = quote! {
        #[no_mangle]
        fn #config_fn_name() -> #return_type {
            #[inline(always)]
            #config_function

            #config_function_name()
        }
    };

    TokenStream::from(expanded)
}

#[derive(Default)]
struct ConfigAttributes {
    kind: Option<ConfigKind>,
}

impl ConfigAttributes {
    fn parse(&mut self, meta: &syn::meta::ParseNestedMeta) -> syn::Result<()> {
        use enum_iterator::all;

        for (config_name, kind) in all::<ConfigKind>().map(|c| (c.as_name(), c)) {
            if meta.path.is_ident(config_name) {
                self.check_only_one_kind(config_name);
                self.kind = Some(kind);
                return Ok(());
            }
        }

        let supported_params = all::<ConfigKind>()
            .map(|c| format!("`{}`", c.as_name()))
            .collect::<Vec<_>>()
            .join(", ");
        Err(meta.error(format!(
            "unsupported parameter ({supported_params} are supported)",
        )))
    }

    fn check_only_one_kind(&self, param: &str) {
        assert!(
            self.kind.is_none(),
            "a separate function is required for `{param}` configuration",
        );
    }
}

#[derive(Debug, enum_iterator::Sequence)]
enum ConfigKind {
    Network,
    Usb,
}

impl ConfigKind {
    fn as_name(&self) -> &'static str {
        match self {
            Self::Network => "network",
            Self::Usb => "usb",
        }
    }
}
