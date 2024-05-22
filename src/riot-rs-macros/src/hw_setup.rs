#[proc_macro_attribute]
pub fn hw_setup(_args: TokenStream, _item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::HwSetup;

    // FIXME: check that the item is indeed just a module declaration, and reuse its name
    let mod_name = format_ident!("sensors");

    let riot_rs_crate = utils::riot_rs_crate();

    let hwsetup_path = std::path::PathBuf::from(std::env::var("SETUP_FILE").unwrap());
    let hwsetup = HwSetup::read_from_path(&hwsetup_path).unwrap();
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
    use riot_rs_hwsetup::{
        peripherals::PullResistor,
        sensors::{Sensor, SensorBus, SensorConfig},
    };
    use serde_yaml::Value as YamlValue;

    use crate::utils;

    pub fn generate_sensor(riot_rs_crate: &syn::Ident, sensor_setup: &Sensor) -> TokenStream {
        let sensor_name = sensor_setup.name();

        let sensor_name_static = format_ident!("{sensor_name}");
        let sensor_ref = format_ident!("{sensor_name}_REF");
        let sensor_type = utils::parse_type_path(sensor_setup.driver());
        let sensor_label = sensor_setup.label();

        // Path of the module containing the sensor driver
        let sensor_mod = utils::parse_parent_module_path(sensor_setup.driver());
        let sensor_mod = utils::parse_type_path(&sensor_mod);

        let spawner_fn = format_ident!("{sensor_name}_init");

        let cfg_conds = utils::parse_cfg_conditionals(sensor_setup);

        let one_shot_peripheral_struct_ident = format_ident!("{sensor_name}Peripherals");
        let mut use_one_shot_peripheral_struct = false;
        let mut peripheral = None;

        // FIXME: these are not mutually exclusive
        let sensor_init = if let Some(SensorBus::I2c(i2cs)) = sensor_setup.bus() {
            // FIXME: handle conds
            let bus_name = i2cs.keys().next().unwrap();
            let i2c_bus_static = format_ident!("{}", super::i2c_bus_static(bus_name));

            // FIXME: maybe do not even pass raw peripherals, always wrap them into embedded_hal
            // types/arch types (including the internal temp sensor)
            // TODO: select the appropriate I2C instance
            quote! {
                let i2c_bus = #riot_rs_crate::embassy::#i2c_bus_static.get().unwrap();
                let i2c_dev = #riot_rs_crate::embassy::arch::i2c::I2cDevice::new(i2c_bus);
                #sensor_name_static.init(spawner, peripherals, i2c_dev, config);
            }
        } else if let Some(peripherals) = sensor_setup.peripherals() {
            // FIXME: handle multiple GPIOs
            // FIXME: handle multiple peripheral types
            let input = peripherals.inputs().next().unwrap();

            // TODO: move this match elsewhere?
            let pull_setting = match input.pull() {
                PullResistor::Up => quote! { Up },
                PullResistor::Down => quote! { Down },
                PullResistor::None => quote! { None },
            };

            peripheral = Some(format_ident!("{}", input.pin()));

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
        };

        let (peripheral_struct, one_shot_peripheral_struct) = if use_one_shot_peripheral_struct {
            let peripheral = peripheral.unwrap();

            (
                quote! { #one_shot_peripheral_struct_ident },
                quote! {
                    #riot_rs_crate::embassy::define_peripherals!(#one_shot_peripheral_struct_ident {
                        p: #peripheral
                    });
                },
            )
        } else {
            (quote! { #sensor_mod::Peripherals }, quote! {})
        };

        let sensor_config = generate_sensor_config(sensor_setup.with());

        let expanded = quote! {
            // Instantiate the sensor driver
            // FIXME: does this work with zero cfg_conds?
            #[cfg(all(#(#cfg_conds),*))]
            pub static #sensor_name_static: #sensor_type = #sensor_type::new(#sensor_label);

            // Store a static reference in the sensor distributed slice
            #[cfg(all(#(#cfg_conds),*))]
            #[#riot_rs_crate::linkme::distributed_slice(#riot_rs_crate::sensors::SENSOR_REFS)]
            #[linkme(crate = #riot_rs_crate::linkme)]
            static #sensor_ref: &'static dyn #riot_rs_crate::sensors::Sensor = &#sensor_name_static;

            // Set the sensor initialization to run at startup
            #[cfg(all(#(#cfg_conds),*))]
            #[#riot_rs_crate::spawner(autostart, peripherals)]
            fn #spawner_fn(spawner: Spawner, peripherals: #peripheral_struct) {
                let mut config = #sensor_mod::Config::default();
                #sensor_config
                #sensor_init
            }

            #one_shot_peripheral_struct
        };

        TokenStream::from(expanded)
    }

    fn generate_sensor_config(with: Option<&SensorConfig>) -> proc_macro2::TokenStream {
        if let Some(with) = with {
            let config_statements = with.iter().map(|(k, v)| {
                let field = format_ident!("{k}");

                let value = match v {
                    YamlValue::String(s) => {
                        if is_type_path_config_string(s) {
                            // NOTE(no-panic): a type path string always has at least two bytes
                            utils::parse_type_path(&s[1..])
                        } else {
                            let s = if s.starts_with('@') {
                                // Discard the first @
                                // NOTE(no-panic): that string has at least one byte as it starts
                                // with @
                                &s[1..]
                            } else {
                                s
                            };
                            quote! { #s }
                        }
                    }
                    YamlValue::Bool(b) => utils::bool_as_token(*b),
                    YamlValue::Number(n) => yaml_number_to_tokens(n),
                    _ => unimplemented!(), // TODO: proper error message
                };

                quote! { config.#field = #value; }
            });

            quote! { #(#config_statements)* }
        } else {
            quote! {}
        }
    }

    fn is_type_path_config_string(string: &str) -> bool {
        string.len() >= 2 && string.starts_with('@') && !string.starts_with("@@")
    }

    fn yaml_number_to_tokens(number: &serde_yaml::Number) -> proc_macro2::TokenStream {
        // TODO: is there a simpler way to do this?
        if number.is_u64() {
            let n = number.as_u64();
            quote! { #n }
        } else if number.is_i64() {
            let n = number.as_i64();
            quote! { #n }
        } else if number.is_f64() {
            let n = number.as_f64();
            quote! { #n }
        } else {
            unimplemented!()
        }
    }
}
