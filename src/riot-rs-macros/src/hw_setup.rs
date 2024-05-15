#[proc_macro_attribute]
pub fn hw_setup(_args: TokenStream, _item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::HwSetup;

    // FIXME: check that the item is indeed just a module declaration, and reuse its name
    let mod_name = format_ident!("sensors");

    let riot_rs_crate = utils::riot_rs_crate();

    let hwsetup = HwSetup::read_from_file().unwrap();
    dbg!(&hwsetup);

    let sensors = hwsetup
        .sensors()
        .iter()
        .map(|sensor| hw_setup::generate_sensor(&riot_rs_crate, sensor));

    let expanded = quote! {
        mod #mod_name {
            use embassy_executor::Spawner;
            use #riot_rs_crate::embassy::arch::peripherals;

            #(#sensors)*
        }
    };

    TokenStream::from(expanded)
}

mod hw_setup {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::{Peripheral, PullResistor, Sensor, SensorBus};

    pub fn generate_sensor(riot_rs_crate: &syn::Ident, sensor_setup: &Sensor) -> TokenStream {
        let sensor_name = sensor_setup.name();

        let sensor_name_static = format_ident!("{sensor_name}");
        let sensor_ref = format_ident!("{sensor_name}_REF");
        let sensor_type = crate::utils::parse_type_path(sensor_setup.driver());

        // Path of the module containing the sensor driver
        // FIXME: is this robust enough?
        let sensor_mod = parse_parent_module_path(sensor_setup.driver());
        let sensor_mod = crate::utils::parse_type_path(&sensor_mod);
        dbg!(&sensor_mod);

        let spawner_fn = format_ident!("{sensor_name}_init");

        let on_conds = parse_conditional_list("context", sensor_setup.on());
        let when_conds = parse_conditional_list("feature", sensor_setup.when());

        // We have to collect the iterator because `cfg_conds` is used multiple times when
        // expanding
        let cfg_conds = on_conds.iter().chain(when_conds.iter()).collect::<Vec<_>>();
        dbg!(&cfg_conds);

        let one_shot_peripheral_struct_ident = format_ident!("{sensor_name}Peripherals");
        let mut use_one_shot_peripheral_struct = false;

        // FIXME: these are not mutually exclusive
        let sensor_init = if let Some(SensorBus::I2c(i2cs)) = sensor_setup.bus() {
            // FIXME: maybe do not even pass raw peripherals, always wrap them into embedded_hal
            // types/arch types (including the internal temp sensor)
            // TODO: select the appropriate I2C instance
            quote! {
                let i2c_bus = #riot_rs_crate::embassy::I2C_BUS.get().unwrap();
                let i2c_dev = #riot_rs_crate::embassy::arch::i2c::I2cDevice::new(i2c_bus);
                #sensor_name_static.init(spawner, peripherals, i2c_dev, config);
            }
        } else {
            if let Some(peripherals) = sensor_setup.peripherals() {
                // FIXME: handle multiple GPIOs
                // FIXME: handle multiple peripheral types
                let input = peripherals.inputs().next().unwrap();

                // TODO: move this match elsewhere?
                let pull_setting = match input.pull() {
                    PullResistor::Up => quote! { Up },
                    PullResistor::Down => quote! { Down },
                    PullResistor::None => quote! { None },
                };

                let peripheral = format_ident!("{}", input.pin());

                use_one_shot_peripheral_struct = true;

                quote! {
                    let pull = #riot_rs_crate::embassy::arch::gpio::Pull::#pull_setting;
                    let input = #riot_rs_crate::embassy::arch::gpio::Input::new(peripherals.p, pull);

                    #sensor_name_static.init(spawner, input, config);
                }
            } else {
                quote! {
                    #sensor_name_static.init(spawner, peripherals, config);
                }
            }

        };

        let (peripheral_struct, one_shot_peripheral_struct) = if use_one_shot_peripheral_struct {
            (
                quote! { #one_shot_peripheral_struct_ident },
                quote! {
                    #riot_rs_crate::embassy::define_peripherals!(#one_shot_peripheral_struct_ident {
                        // p: #peripheral // FIXME
                        p: P0_11, // FIXME
                    });
                }
            )
        } else {
            (
                quote! { #sensor_mod::Peripherals },
                quote! {},
            )
        };

        let expanded = quote! {
            // Instantiate the sensor driver
            // FIXME: does this work with zero cfg_conds?
            #[cfg(all(#(#cfg_conds),*))]
            pub static #sensor_name_static: #sensor_type = #sensor_type::new();

            // Store a static reference in the sensor distributed slice
            #[cfg(all(#(#cfg_conds),*))]
            #[#riot_rs_crate::linkme::distributed_slice(#riot_rs_crate::sensors::SENSOR_REFS)]
            #[linkme(crate = #riot_rs_crate::linkme)]
            static #sensor_ref: &'static dyn #riot_rs_crate::sensors::Sensor = &#sensor_name_static;

            // Set the sensor initialization to run at startup
            #[cfg(all(#(#cfg_conds),*))]
            #[#riot_rs_crate::spawner(autostart, peripherals)]
            fn #spawner_fn(spawner: Spawner, peripherals: #peripheral_struct) {
                let config = #sensor_mod::Config::default();
                // FIXME: set the sensor config from the setup file
                #sensor_init
            }

            #one_shot_peripheral_struct
        };

        TokenStream::from(expanded)
    }

    fn parse_parent_module_path(path: &str) -> String {
        path.split("::<")
            .next()
            .unwrap()
            .rsplit("::")
            .skip(1)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("::")
    }

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
