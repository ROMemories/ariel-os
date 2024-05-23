#[proc_macro_attribute]
pub fn hw_setup_init(_args: TokenStream, _item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::HwSetup;

    // TODO: check that the item is indeed just a function declaration
    let fn_name = format_ident!("codegened_init");

    let hwsetup_path = HwSetup::get_path_from_env().unwrap();
    let hwsetup = HwSetup::read_from_path(&hwsetup_path).unwrap();

    let i2c_buses = hw_setup_init::generate_i2c_bus(hwsetup.buses().i2c());
    let spi_buses = hw_setup_init::generate_spi_bus(hwsetup.buses().spi());

    let buses = i2c_buses.into_iter().chain(spi_buses);
    let bus_instantiations = buses.clone().map(hw_setup_init::Bus::into_instantiation);
    let bus_initializations = buses.flat_map(hw_setup_init::Bus::into_initializations);

    let expanded = quote! {
        #(#bus_instantiations)*

        fn codegened_init(peripherals: &mut arch::OptionalPeripherals) {
            #(#bus_initializations)*
        }
    };

    TokenStream::from(expanded)
}

fn i2c_bus_static(bus_label: &str) -> String {
    format!("I2C_BUS_{bus_label}")
}

fn spi_bus_static(bus_label: &str) -> String {
    format!("SPI_BUS_{bus_label}")
}

mod hw_setup_init {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::buses;

    use crate::utils;

    #[derive(Debug, Clone)]
    pub struct Bus {
        instantiation: TokenStream,
        initializations: Vec<TokenStream>,
    }

    impl Bus {
        pub fn into_instantiation(self) -> TokenStream {
            self.instantiation
        }

        pub fn into_initializations(self) -> Vec<TokenStream> {
            self.initializations
        }
    }

    pub fn generate_i2c_bus(i2c_setup: &[buses::i2c::Bus]) -> Vec<Bus> {
        use buses::i2c::Frequency;

        i2c_setup
            .iter()
            .map(|bus| {
                let i2c_bus_static = format_ident!("{}", super::i2c_bus_static(bus.name()));

                let instantiation = quote! {
                    // Using the OnceCell from once_cell instead of the one from core because it supports
                    // critical sections.
                    // TODO: move this to a `bus` module?
                    pub static #i2c_bus_static: once_cell::sync::OnceCell<
                        embassy_sync::mutex::Mutex<
                            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                            arch::i2c::I2c,
                        >,
                    > = once_cell::sync::OnceCell::new();
                };

                let initializations = bus
                    .peripheral()
                    .iter()
                    .map(|(name, peripheral)| {
                        let cfg_conds = crate::utils::parse_cfg_conditionals(peripheral);

                        // TODO: is this the best place to do this conversion?
                        let frequency = match peripheral.frequency() {
                            Frequency::K100 => quote! { arch::i2c::Frequency::K100 },
                            Frequency::K250 => quote! { arch::i2c::Frequency::K250 },
                            Frequency::K400 => quote! { arch::i2c::Frequency::K400 },
                        };

                        // FIXME: support on/when on sda/scl
                        let sda_pullup =
                            utils::bool_as_token(peripheral.sda().first().unwrap().pull_up());
                        let scl_pullup =
                            utils::bool_as_token(peripheral.scl().first().unwrap().pull_up());

                        // FIXME: test what happens when trying to use a peripheral that doesn't exist or that is
                        // already used
                        // FIXME: support on/when on sda/scl
                        let sda_peripheral =
                            format_ident!("{}", peripheral.sda().first().unwrap().pin());
                        let scl_peripheral =
                            format_ident!("{}", peripheral.scl().first().unwrap().pin());

                        let i2c_peripheral = format_ident!("{name}");

                        quote! {
                            #[cfg(all(#(#cfg_conds),*))]
                            {
                                let mut config = arch::i2c::Config::default();
                                config.frequency = #frequency;
                                config.sda_pullup = #sda_pullup;
                                config.scl_pullup = #scl_pullup;
                                // FIXME: use configuration
                                config.sda_high_drive = false;
                                config.scl_high_drive = false;

                                let i2c = arch::i2c::I2c::new(
                                    peripherals.#i2c_peripheral.take().unwrap(),
                                    peripherals.#sda_peripheral.take().unwrap(),
                                    peripherals.#scl_peripheral.take().unwrap(),
                                    config,
                                );

                                let _ = #i2c_bus_static.set(embassy_sync::mutex::Mutex::new(i2c));
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                Bus {
                    instantiation,
                    initializations,
                }
            })
            .collect::<Vec<_>>()
    }

    pub fn generate_spi_bus(spi_setup: &[buses::spi::Bus]) -> Vec<Bus> {
        spi_setup
            .iter()
            .map(|bus| {
                let spi_bus_static = format_ident!("{}", super::spi_bus_static(bus.name()));

                let instantiation = quote! {
                    // Using the OnceCell from once_cell instead of the one from core because it supports
                    // critical sections.
                    pub static #spi_bus_static: once_cell::sync::OnceCell<
                        embassy_sync::mutex::Mutex<
                            embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                            arch::spi::Spi,
                        >,
                    > = once_cell::sync::OnceCell::new();
                };

                let initializations = bus
                    .peripheral()
                    .iter()
                    .map(|(name, peripheral)| {
                        let cfg_conds = crate::utils::parse_cfg_conditionals(peripheral);

                        // FIXME: test what happens when trying to use a peripheral that doesn't exist or that is
                        // already used
                        // FIXME: support on/when on sck/miso/mosi
                        let sck_peripheral =
                            format_ident!("{}", peripheral.sck().first().unwrap().pin());
                        let miso_peripheral =
                            format_ident!("{}", peripheral.miso().first().unwrap().pin());
                        let mosi_peripheral =
                            format_ident!("{}", peripheral.mosi().first().unwrap().pin());

                        let spi_peripheral = format_ident!("{name}");

                        quote! {
                            #[cfg(all(#(#cfg_conds),*))]
                            {
                                let mut config = arch::spi::Config::default();
                                // FIXME: set the config

                                let spi = arch::spi::Spi::new(
                                    peripherals.#spi_peripheral.take().unwrap(),
                                    peripherals.#sck_peripheral.take().unwrap(),
                                    peripherals.#miso_peripheral.take().unwrap(),
                                    peripherals.#mosi_peripheral.take().unwrap(),
                                    config,
                                );

                                let _ = #spi_bus_static.set(embassy_sync::mutex::Mutex::new(spi));
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                Bus {
                    instantiation,
                    initializations,
                }
            })
            .collect::<Vec<_>>()
    }
}
