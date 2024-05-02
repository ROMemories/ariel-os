#[proc_macro_attribute]
pub fn hw_setup_init(_args: TokenStream, _item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::HwSetup;

    // FIXME: check that the item is indeed just a function declaration
    let fn_name = format_ident!("codegened_init");

    let hwsetup = HwSetup::read_from_file().unwrap();
    dbg!(&hwsetup);

    let i2c_buses = hwsetup
        .buses()
        .i2c()
        .iter()
        .map(|bus| hw_setup_init::generate_i2c_bus_init(bus));
    // TODO: chain other buses
    let buses = i2c_buses;

    let expanded = quote! {
        fn codegened_init(peripherals: &mut arch::OptionalPeripherals) {
            #(#buses)*
        }
    };

    TokenStream::from(expanded)
}

mod hw_setup_init {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::{I2cBus, I2cFrequency};

    use crate::utils;

    pub fn generate_i2c_bus_init(i2c_setup: &I2cBus) -> TokenStream {
        let on_conds = parse_conditional_list("context", i2c_setup.on());
        let when_conds = parse_conditional_list("feature", i2c_setup.when());

        let cfg_conds = on_conds.iter().chain(when_conds.iter()).collect::<Vec<_>>();

        // TODO: is this the best place to do this conversion?
        let frequency = match i2c_setup.frequency() {
            I2cFrequency::K100 => quote! { arch::i2c::Frequency::K100 },
            I2cFrequency::K250 => quote! { arch::i2c::Frequency::K250 },
            I2cFrequency::K400 => quote! { arch::i2c::Frequency::K400 },
        };

        // FIXME: support on/when on sda/scl
        let sda_pullup = utils::bool_as_token(i2c_setup.sda().first().unwrap().pull_up());
        let scl_pullup = utils::bool_as_token(i2c_setup.scl().first().unwrap().pull_up());

        // FIXME: test what happens when trying to use a peripheral that doesn't exist or that is
        // already used
        // FIXME: support on/when on sda/scl
        let sda_peripheral = format_ident!("{}", i2c_setup.sda().first().unwrap().pin());
        let scl_peripheral = format_ident!("{}", i2c_setup.scl().first().unwrap().pin());
        dbg!(&sda_peripheral, &scl_peripheral);

        let expanded = quote! {
            #[cfg(all(#(#cfg_conds),*))]
            {
                let mut config = arch::i2c::Config::default();
                config.frequency = #frequency;
                config.sda_pullup = #sda_pullup;
                config.scl_pullup = #scl_pullup;
                // FIXME: use configuration
                config.sda_high_drive = false;
                config.scl_high_drive = false;

                // FIXME: use configuration
                let i2c = arch::i2c::I2c::new(
                    peripherals.TWISPI0.take().unwrap(),
                    peripherals.#sda_peripheral.take().unwrap(),
                    peripherals.#scl_peripheral.take().unwrap(),
                    config,
                );

                let i2c_bus = embassy_sync::mutex::Mutex::new(i2c);
                let _ = I2C_BUS.set(i2c_bus);
            }
        };

        TokenStream::from(expanded)
    }

    // FIXME: factor this out with hw_setup
    fn parse_conditional_list(cfg_attr: &str, conditionals: Option<&str>) -> Vec<TokenStream> {
        if let Some(on) = conditionals {
            let context_attr = format_ident!("{cfg_attr}");

            on.split(',')
                .map(str::trim)
                .map(|context| quote!(#context_attr = #context))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    }
}
