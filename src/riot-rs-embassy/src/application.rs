use crate::{arch, DefinePeripheralsError, Drivers};

#[cfg(feature = "usb")]
use embassy_usb::Builder as UsbBuilder;

#[cfg(feature = "usb")]
use arch::usb::UsbDriver;

/// Defines an application.
///
/// Allows to separate its fallible initialization from its infallible running phase.
pub trait Application {
    fn init() -> &'static dyn Application
    where
        Self: Sized;

    #[cfg(feature = "usb")]
    fn usb_builder_hook(&self, _usb_builder: &mut UsbBuilder<'static, UsbDriver>) {}

    /// Applications must implement this to obtain the peripherals they require.
    ///
    /// This function is only run once at startup and instantiates the application.
    /// No guarantee is provided regarding the order in which different applications are
    /// initialized.
    /// The [`define_peripherals!`](crate::define_peripherals!) macro can be leveraged to extract
    /// the required peripherals.
    /// This function must not block but may spawn [Embassy tasks](embassy_executor::task) using
    /// the provided [`Spawner`](embassy_executor::Spawner).
    /// In addition, it is provided with the drivers initialized by the system.
    fn start(
        &self,
        peripherals: &mut arch::OptionalPeripherals,
        spawner: embassy_executor::Spawner,
        drivers: Drivers,
    ) -> Result<(), ApplicationError>;
}

/// Represents errors that can happen during application initialization.
#[derive(Debug)]
pub enum ApplicationError {
    /// The application could not obtain a peripheral, most likely because it was already used by
    /// another application or by the system itself.
    CannotTakePeripheral,
}

impl From<DefinePeripheralsError> for ApplicationError {
    fn from(err: DefinePeripheralsError) -> Self {
        match err {
            DefinePeripheralsError::TakingPeripheral => Self::CannotTakePeripheral,
        }
    }
}

/// Sets the [`Application::initialize()`] function implemented on the provided type to be run at
/// startup.
#[macro_export]
macro_rules! riot_initialize {
    ($prog_type:ident) => {
        #[$crate::distributed_slice($crate::EMBASSY_TASKS)]
        #[linkme(crate = $crate::linkme)]
        fn __init_application() -> &'static dyn $crate::application::Application {
            <$prog_type as Application>::init()
        }
    };
}
