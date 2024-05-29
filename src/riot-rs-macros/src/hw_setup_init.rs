#[proc_macro_attribute]
pub fn hw_setup_init(_args: TokenStream, _item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::HwSetup;

    // TODO: check that the item is indeed just a function declaration
    let fn_name = format_ident!("codegened_init");

    // Allow tooling to run without passing a setup file
    let hwsetup = if let Ok(hwsetup_path) = HwSetup::get_path_from_env() {
        HwSetup::read_from_path(&hwsetup_path).unwrap()
    } else {
        HwSetup::default()
    };

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

                        let frequency = i2c_frequency(peripheral.frequency());

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

    fn i2c_frequency(frequency: buses::i2c::Frequency) -> TokenStream {
        use buses::i2c::Frequency;

        // TODO: is this the best place to do this conversion?
        match frequency {
            Frequency::K100 => quote! { arch::i2c::Frequency::K100 },
            Frequency::K250 => quote! { arch::i2c::Frequency::K250 },
            Frequency::K400 => quote! { arch::i2c::Frequency::K400 },
        }
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

                        let frequency = spi_frequency(peripheral.frequency());
                        let mode = spi_mode(peripheral.mode());
                        let bit_order = spi_bit_order(peripheral.bit_order());

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
                                config.frequency = #frequency;
                                config.mode = #mode;
                                config.bit_order = #bit_order;
                                // TODO: driver strengths

                                // FIXME: support on/when on sck/miso/mosi
                                let spi_peripheral = peripherals.#spi_peripheral.take().unwrap();
                                let sck_peripheral = peripherals.#sck_peripheral.take().unwrap();
                                let miso_peripheral = peripherals.#miso_peripheral.take().unwrap();
                                let mosi_peripheral = peripherals.#mosi_peripheral.take().unwrap();

                                // FIXME: make sure that the order MISO/MOSI/SCK is the same for
                                // all archs
                                let spi = arch::spi::Spi::new(
                                    spi_peripheral,
                                    sck_peripheral,
                                    miso_peripheral,
                                    mosi_peripheral,
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

    fn spi_frequency(frequency: buses::spi::Frequency) -> TokenStream {
        use buses::spi::Frequency;

        // TODO: is this the best place to do this conversion?
        match frequency {
            Frequency::K125 => quote! { arch::spi::Frequency::K125 },
            Frequency::K250 => quote! { arch::spi::Frequency::K250 },
            Frequency::K500 => quote! { arch::spi::Frequency::K500 },
            Frequency::M1 => quote! { arch::spi::Frequency::M1 },
            Frequency::M2 => quote! { arch::spi::Frequency::M2 },
            Frequency::M4 => quote! { arch::spi::Frequency::M4 },
            Frequency::M8 => quote! { arch::spi::Frequency::M8 },
            Frequency::M16 => quote! { arch::spi::Frequency::M16 },
            Frequency::M32 => quote! { arch::spi::Frequency::M32 },
        }
    }

    fn spi_mode(mode: buses::spi::Mode) -> TokenStream {
        use buses::spi::Mode;

        match mode {
            Mode::Mode0 => quote! { arch::spi::MODE_0 },
            Mode::Mode1 => quote! { arch::spi::MODE_1 },
            Mode::Mode2 => quote! { arch::spi::MODE_2 },
            Mode::Mode3 => quote! { arch::spi::MODE_3 },
        }
    }

    fn spi_bit_order(bit_order: buses::spi::BitOrder) -> TokenStream {
        use buses::spi::BitOrder;

        match bit_order {
            BitOrder::MsbFirst => quote! { arch::spi::BitOrder::MSB_FIRST },
            BitOrder::LsbFirst => quote! { arch::spi::BitOrder::LSB_FIRST },
        }
    }
}
