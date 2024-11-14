//! Provides network access.
//!
//! The network link to use is selected through Cargo features.
//! Additionally, the [`riot_rs::config`](riot_rs_macros::config) attribute macro allows to provide
//! custom network configuration.

#![deny(missing_docs)]

use embassy_net::{Runner, Stack};
use embassy_sync::once_lock::OnceLock;

use crate::{sendcell::SendCell, NetworkDevice};

#[allow(dead_code)]
pub(crate) const ETHERNET_MTU: usize = 1514;

/// A network stack.
///
/// Required to create a UDP or TCP socket.
#[expect(clippy::module_name_repetitions)]
pub type NetworkStack = Stack<'static>;

pub(crate) static STACK: OnceLock<SendCell<NetworkStack>> = OnceLock::new();

/// Returns a new [`NetworkStack`].
///
/// Returns [`None`] if networking is not yet initialized.
#[expect(clippy::module_name_repetitions)]
pub async fn network_stack() -> Option<NetworkStack> {
    STACK.get().await.get_async().await.copied()
}

#[embassy_executor::task]
pub(crate) async fn net_task(mut runner: Runner<'static, NetworkDevice>) -> ! {
    runner.run().await
}

#[allow(dead_code, reason = "false positive during builds outside of laze")]
pub(crate) fn config() -> embassy_net::Config {
    #[cfg(not(feature = "override-network-config"))]
    {
        embassy_net::Config::dhcpv4(embassy_net::DhcpConfig::default())
    }
    #[cfg(feature = "override-network-config")]
    {
        extern "Rust" {
            fn riot_rs_network_config() -> embassy_net::Config;
        }
        unsafe { riot_rs_network_config() }
    }
}

/// Constructor for [`DummyDriver`]
///
/// This is a standalone function instead of an associated method to ease moving [`DummyDriver`]
/// into [`embassy_net`].
#[allow(
    dead_code,
    reason = "constructor is only used in linter / documentation situations"
)]
pub(crate) fn new_dummy() -> DummyDriver {
    unimplemented!();
}

/// Stand-in for a network driver in documentation and linting.
///
/// It also doubles as the infallible type for its own associated types.
// FIXME: This should be core::convert::Infallible as soon as embassy-net implements the traits on
// that.
pub(crate) struct DummyDriver(core::convert::Infallible);

impl embassy_net::driver::Driver for DummyDriver {
    type RxToken<'a> = Self
    where
        Self: 'a;

    type TxToken<'a> = Self
    where
        Self: 'a;

    fn receive(
        &mut self,
        _cx: &mut core::task::Context,
    ) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        match self.0 {}
    }

    fn transmit(&mut self, _cx: &mut core::task::Context) -> Option<Self::TxToken<'_>> {
        match self.0 {}
    }

    fn link_state(&mut self, _cx: &mut core::task::Context) -> embassy_net::driver::LinkState {
        match self.0 {}
    }

    fn capabilities(&self) -> embassy_net::driver::Capabilities {
        match self.0 {}
    }

    fn hardware_address(&self) -> embassy_net::driver::HardwareAddress {
        match self.0 {}
    }
}

impl embassy_net::driver::TxToken for DummyDriver {
    fn consume<R, F>(self, _len: usize, _f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        match self.0 {}
    }
}

impl embassy_net::driver::RxToken for DummyDriver {
    fn consume<R, F>(self, _f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        match self.0 {}
    }
}
