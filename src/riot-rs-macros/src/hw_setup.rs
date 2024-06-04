#[proc_macro_attribute]
pub fn hw_setup(_args: TokenStream, _item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::HwSetup;

    // FIXME: check that the item is indeed just a module declaration, and reuse its name
    let mod_name = format_ident!("sensors");

    let riot_rs_crate = utils::riot_rs_crate();

    let hwsetup_path = HwSetup::get_path_from_env().unwrap();
    let hwsetup = HwSetup::read_from_path(&hwsetup_path).unwrap();
    dbg!(&hwsetup);

    let sensors = hwsetup
        .sensors()
        .connected()
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
        sensors::{Sensor, SensorBus, SensorConfig, SensorConfigValue, StringOrTypePath, YamlNumber},
    };

    use crate::utils;

    pub fn generate_sensor(riot_rs_crate: &syn::Ident, sensor_setup: &Sensor) -> TokenStream {
        let sensor_name = sensor_setup.name();

        let sensor_name_static = format_ident!("{sensor_name}");
        let sensor_ref = format_ident!("{sensor_name}_REF");
        let sensor_label = sensor_setup.label().unwrap(); // FIXME: pass the Option to the sensor
                                                          // directly

        let driver = match StringOrTypePath::from(sensor_setup.driver()) {
            StringOrTypePath::TypePath(type_path) => type_path,
            _ => panic!("`driver` must start with an @"),
        };
        let sensor_type = utils::parse_type_path(driver);
        let sensor_type_alias_name = sensor_type_alias_name(sensor_name, driver);
        // Path of the module containing the sensor driver
        let sensor_mod = utils::parse_type_path(&utils::parse_parent_module_path(driver));

        let spawner_fn = format_ident!("{sensor_name}_init");

        let mut peripheral_defs = Vec::new();
        let mut sensor_inits = Vec::new();

        // A sensor can only use one bus
        if let Some(SensorBus::I2c(i2cs)) = sensor_setup.bus() {
            // FIXME: handle conds
            let bus_name = i2cs.keys().next().unwrap();
            let i2c_bus_static = format_ident!("{}", super::i2c_bus_static(bus_name));

            // TODO: select the appropriate I2C instance
            let i2c_init = quote! {
                let i2c_bus = #riot_rs_crate::embassy::#i2c_bus_static.get().unwrap();
                let i2c_dev = #riot_rs_crate::embassy::arch::i2c::I2cDevice::new(i2c_bus);
                #sensor_name_static.init(spawner, peripherals, i2c_dev, config);
            };
            sensor_inits.push(i2c_init);
        } else if let Some(SensorBus::Spi(spis)) = sensor_setup.bus() {
            // FIXME: handle conds
            let bus_name = spis.keys().next().unwrap();
            let spi_bus_static = format_ident!("{}", super::spi_bus_static(bus_name));

            for cs in spis.values().next().unwrap().chip_selects() {
                let cs_cfg_conds = utils::parse_cfg_conditionals(cs);
                let cs_pin = format_ident!("{}", cs.pin());
                peripheral_defs.push(quote! {
                    #[cfg(all(#(#cs_cfg_conds),*))]
                    p: #cs_pin
                });
            }

            // TODO: select the appropriate SPI instance
            let spi_init = quote! {
                let spi_bus = #riot_rs_crate::embassy::#spi_bus_static.get().unwrap();
                // FIXME: depends on the arch (e.g., rp)
                let cs_output = #riot_rs_crate::embassy::arch::gpio::Output::new(
                    peripherals.p,
                    #riot_rs_crate::embassy::arch::gpio::Level::High,
                    #[cfg(context = "nrf")]
                    #riot_rs_crate::embassy::arch::gpio::OutputDrive::Standard,
                );
                let spi_dev = #riot_rs_crate::embassy::arch::spi::SpiDevice::new(spi_bus, cs_output);

                #sensor_name_static.init(spawner, spi_dev, config);
            };
            sensor_inits.push(spi_init);
        }

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

            let input_pin = format_ident!("{}", input.pin());
            peripheral_defs.push(quote! { p: #input_pin });

            let gpio_init = quote! {
                let pull = #riot_rs_crate::embassy::arch::gpio::Pull::#pull_setting;
                let input = #riot_rs_crate::embassy::arch::gpio::Input::new(peripherals.p, pull);

                #sensor_name_static.init(spawner, input, config);
            };
            sensor_inits.push(gpio_init);
        }

        if sensor_inits.is_empty() {
            let only_init = quote! {
                #sensor_name_static.init(spawner, peripherals, config);
            };
            sensor_inits.push(only_init);
        }

        let cfg_conds = utils::parse_cfg_conditionals(sensor_setup);

        let (peripheral_struct_path, one_shot_peripheral_struct) = if !peripheral_defs.is_empty() {
            let one_shot_peripheral_struct_ident = format_ident!("{sensor_name}Peripherals");

            // TODO: make this work for multiple peripherals, with a HashMap
            (
                quote! { #one_shot_peripheral_struct_ident },
                quote! {
                    #[cfg(all(#(#cfg_conds),*))]
                    #riot_rs_crate::embassy::define_peripherals!(#one_shot_peripheral_struct_ident {
                        #(#peripheral_defs),*
                    });
                },
            )
        } else {
            (quote! { #sensor_mod::Peripherals }, quote! {})
        };

        let sensor_config = generate_sensor_config(sensor_setup.with());

        let expanded = quote! {
            /// Type alias of this sensor instance
            #[cfg(all(#(#cfg_conds),*))]
            pub type #sensor_type_alias_name = #sensor_type;

            // Instantiate the sensor driver
            // FIXME: does this work with zero cfg_conds?
            #[cfg(all(#(#cfg_conds),*))]
            pub static #sensor_name_static: #sensor_type_alias_name = #sensor_type_alias_name::new(Some(#sensor_label));

            // Store a static reference in the sensor distributed slice
            #[cfg(all(#(#cfg_conds),*))]
            #[#riot_rs_crate::linkme::distributed_slice(#riot_rs_crate::sensors::SENSOR_REFS)]
            #[linkme(crate = #riot_rs_crate::linkme)]
            static #sensor_ref: &'static dyn #riot_rs_crate::sensors::Sensor = &#sensor_name_static;

            // Set the sensor initialization to run at startup
            #[cfg(all(#(#cfg_conds),*))]
            #[#riot_rs_crate::spawner(autostart, peripherals)]
            fn #spawner_fn(spawner: Spawner, peripherals: #peripheral_struct_path) {
                let mut config = #sensor_mod::Config::default();
                #sensor_config

                #(#sensor_inits)*
            }

            #one_shot_peripheral_struct
        };

        TokenStream::from(expanded)
    }

    fn sensor_type_alias_name(sensor_name: &str, driver: &str) -> syn::Ident {
        let sensor_type_name = utils::parse_type_name_from_type_path(driver);
        format_ident!("{sensor_type_name}_{sensor_name}")
    }

    fn generate_sensor_config(with: Option<&SensorConfig>) -> proc_macro2::TokenStream {
        if let Some(with) = with {
            let config_statements = with.iter().map(|(k, v)| {
                let field = format_ident!("{k}");

                let value = match v {
                    SensorConfigValue::String(s) => match StringOrTypePath::from(s) {
                        StringOrTypePath::TypePath(type_path) => utils::parse_type_path(type_path),
                        StringOrTypePath::String(string) => quote! { #string },
                    },
                    SensorConfigValue::Bool(b) => utils::bool_as_token(*b),
                    SensorConfigValue::Number(n) => yaml_number_to_tokens(n),
                };

                quote! { config.#field = #value; }
            });

            quote! { #(#config_statements)* }
        } else {
            quote! {}
        }
    }

    fn yaml_number_to_tokens(number: &YamlNumber) -> proc_macro2::TokenStream {
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
