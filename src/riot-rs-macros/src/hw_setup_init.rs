#[proc_macro_attribute]
pub fn hw_setup_init(_args: TokenStream, _item: TokenStream) -> TokenStream {
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::HwSetup;

    // TODO: check that the item is indeed just a function declaration
    let fn_name = format_ident!("codegened_init");

    let hwsetup_path = std::path::PathBuf::from(std::env::var("SETUP_FILE").unwrap());
    let hwsetup = HwSetup::read_from_path(&hwsetup_path).unwrap();

    let i2c_buses = hwsetup
        .buses()
        .i2c()
        .iter()
        .flat_map(|b| b.peripheral().iter())
        .map(|(peripheral, bus)| hw_setup_init::generate_i2c_bus_init(peripheral, bus));
    // TODO: chain other buses
    let buses = i2c_buses;

    let expanded = quote! {
        fn codegened_init(peripherals: &mut arch::OptionalPeripherals) {
            #(#buses)*
        }

        // Using the OnceCell from once_cell instead of the one from core because it supports
        // critical sections.
        // TODO: move this to a `bus` module?
        pub static I2C_BUS: once_cell::sync::OnceCell<
            embassy_sync::mutex::Mutex<
                embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                arch::i2c::I2c,
            >,
        > = once_cell::sync::OnceCell::new();

        // Using the OnceCell from once_cell instead of the one from core because it supports
        // critical sections.
        pub static SPI_BUS: once_cell::sync::OnceCell<
            embassy_sync::mutex::Mutex<
                embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
                arch::spi::Spi,
            >,
        > = once_cell::sync::OnceCell::new();
    };

    TokenStream::from(expanded)
}

mod hw_setup_init {
    use proc_macro2::TokenStream;
    use quote::{format_ident, quote};
    use riot_rs_hwsetup::buses;

    use crate::utils;

    pub fn generate_i2c_bus_init(
        peripheral: &str,
        i2c_setup: &buses::i2c::BusPeripheral,
    ) -> TokenStream {
        use buses::i2c::Frequency;

        let cfg_conds = crate::utils::parse_cfg_conditionals(i2c_setup);

        // TODO: is this the best place to do this conversion?
        let frequency = match i2c_setup.frequency() {
            Frequency::K100 => quote! { arch::i2c::Frequency::K100 },
            Frequency::K250 => quote! { arch::i2c::Frequency::K250 },
            Frequency::K400 => quote! { arch::i2c::Frequency::K400 },
        };

        // FIXME: support on/when on sda/scl
        let sda_pullup = utils::bool_as_token(i2c_setup.sda().first().unwrap().pull_up());
        let scl_pullup = utils::bool_as_token(i2c_setup.scl().first().unwrap().pull_up());

        // FIXME: test what happens when trying to use a peripheral that doesn't exist or that is
        // already used
        // FIXME: support on/when on sda/scl
        let sda_peripheral = format_ident!("{}", i2c_setup.sda().first().unwrap().pin());
        let scl_peripheral = format_ident!("{}", i2c_setup.scl().first().unwrap().pin());

        let i2c_peripheral = format_ident!("{peripheral}");

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

                let i2c = arch::i2c::I2c::new(
                    peripherals.#i2c_peripheral.take().unwrap(),
                    peripherals.#sda_peripheral.take().unwrap(),
                    peripherals.#scl_peripheral.take().unwrap(),
                    config,
                );

                let _ = I2C_BUS.set(embassy_sync::mutex::Mutex::new(i2c));
            }
        };

        TokenStream::from(expanded)
    }

    pub fn generate_spi_bus_init(
        peripheral: &str,
        spi_setup: &buses::spi::BusPeripheral,
    ) -> TokenStream {
        let cfg_conds = crate::utils::parse_cfg_conditionals(spi_setup);

        // FIXME: test what happens when trying to use a peripheral that doesn't exist or that is
        // already used
        // FIXME: support on/when on sck/miso/mosi
        let sck_peripheral = format_ident!("{}", spi_setup.sck().first().unwrap().pin());
        let miso_peripheral = format_ident!("{}", spi_setup.miso().first().unwrap().pin());
        let mosi_peripheral = format_ident!("{}", spi_setup.mosi().first().unwrap().pin());

        let spi_peripheral = format_ident!("{peripheral}");

        let expanded = quote! {
            #[cfg(all(#(#cfg_conds),*))]
            {
                let mut config = arch::spi::Config::default();
                // FIXME: set the config

                let spi = arch::spi::Spi::new(
                    peripherals.#sck_peripheral.take().unwrap(),
                    peripherals.#miso_peripheral.take().unwrap(),
                    peripherals.#mosi_peripheral.take().unwrap(),
                    config,
                );

                let _ = SPI_BUS.set(embassy_sync::mutex::Mutex::new(spi));
            }
        };

        TokenStream::from(expanded)
    }
}
