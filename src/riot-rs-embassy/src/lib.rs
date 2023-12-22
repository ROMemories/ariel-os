#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use core::{cell::OnceCell, ops::DerefMut};

use linkme::distributed_slice;
use static_cell::make_static;

use embassy_executor::{InterruptExecutor, Spawner};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

#[cfg(feature = "usb")]
use embassy_usb::{Builder as UsbBuilder, UsbDevice};

pub mod blocker;

pub type Task = fn(
    &mut arch::OptionalPeripherals,
    InitializationArgs,
) -> Result<&dyn UserProgram, ProgramInitError>;

// TODO: rename this
#[derive(Copy, Clone)]
pub struct InitializationArgs {
    pub peripherals: &'static Mutex<CriticalSectionRawMutex, arch::OptionalPeripherals>,
}

// TODO: rename this
#[derive(Copy, Clone)]
pub struct Drivers {
    #[cfg(feature = "usb_ethernet")]
    pub stack: &'static OnceCell<&'static UsbEthernetStack>,
}

pub type UsbEthernetStack = Stack<UsbEthernetDevice<'static, ETHERNET_MTU>>;

pub static EXECUTOR: InterruptExecutor = InterruptExecutor::new();

#[distributed_slice]
pub static EMBASSY_TASKS: [Task] = [..];

#[cfg(context = "nrf52")]
mod nrf52 {
    pub use embassy_nrf::interrupt;
    pub use embassy_nrf::interrupt::SWI0_EGU0 as SWI;
    pub use embassy_nrf::{init, peripherals, OptionalPeripherals};

    #[cfg(feature = "usb")]
    use embassy_nrf::{bind_interrupts, rng, usb as nrf_usb};

    #[cfg(feature = "usb")]
    bind_interrupts!(struct Irqs {
        USBD => nrf_usb::InterruptHandler<peripherals::USBD>;
        POWER_CLOCK => nrf_usb::vbus_detect::InterruptHandler;
        RNG => rng::InterruptHandler<peripherals::RNG>;
    });

    #[interrupt]
    unsafe fn SWI0_EGU0() {
        crate::EXECUTOR.on_interrupt()
    }

    #[cfg(feature = "usb")]
    pub mod usb {
        use embassy_nrf::peripherals;
        use embassy_nrf::usb::{vbus_detect::HardwareVbusDetect, Driver};
        pub type UsbDriver = Driver<'static, peripherals::USBD, HardwareVbusDetect>;
        pub fn driver(usbd: peripherals::USBD) -> UsbDriver {
            use super::Irqs;
            Driver::new(usbd, Irqs, HardwareVbusDetect::new(Irqs))
        }
    }
}

#[cfg(context = "rp2040")]
mod rp2040 {
    pub use embassy_rp::interrupt;
    pub use embassy_rp::interrupt::SWI_IRQ_1 as SWI;
    pub use embassy_rp::{init, peripherals, OptionalPeripherals};

    #[cfg(feature = "usb")]
    use embassy_rp::{bind_interrupts, peripherals::USB, usb::InterruptHandler};

    // rp2040 usb start
    #[cfg(feature = "usb")]
    bind_interrupts!(struct Irqs {
        USBCTRL_IRQ => InterruptHandler<USB>;
    });

    #[interrupt]
    unsafe fn SWI_IRQ_1() {
        crate::EXECUTOR.on_interrupt()
    }

    #[cfg(feature = "usb")]
    pub mod usb {
        use embassy_rp::peripherals;
        use embassy_rp::usb::Driver;
        pub type UsbDriver = Driver<'static, peripherals::USB>;
        pub fn driver(usb: peripherals::USB) -> UsbDriver {
            Driver::new(usb, super::Irqs)
        }
    }
}

#[cfg(context = "nrf52")]
use nrf52 as arch;

#[cfg(context = "rp2040")]
use rp2040 as arch;

use arch::SWI;

//
// usb common start
#[cfg(feature = "usb")]
use arch::usb::UsbDriver;

#[cfg(feature = "usb")]
#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, UsbDriver>) -> ! {
    device.run().await
}
// usb common end
//

//
// net begin
const ETHERNET_MTU: usize = 1514;

#[cfg(feature = "net")]
use embassy_net::{Stack, StackResources};
// net end
//

//
// usb net begin
#[cfg(feature = "usb_ethernet")]
use embassy_usb::class::cdc_ncm::embassy_net::{
    Device as UsbEthernetDevice, Runner as UsbEthernetRunner,
};

#[cfg(feature = "usb_ethernet")]
#[embassy_executor::task]
async fn usb_ncm_task(class: UsbEthernetRunner<'static, UsbDriver, ETHERNET_MTU>) -> ! {
    class.run().await
}

#[cfg(feature = "usb_ethernet")]
#[embassy_executor::task]
async fn usb_ethernet_task(stack: &'static UsbEthernetStack) -> ! {
    stack.run().await
}
// usb net end
//

#[cfg(feature = "usb")]
const fn usb_default_config() -> embassy_usb::Config<'static> {
    // Create embassy-usb Config
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("USB-Ethernet example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // Required for Windows support.
    config.composite_with_iads = true;
    config.device_class = 0xEF;
    config.device_sub_class = 0x02;
    config.device_protocol = 0x01;
    config
}

#[distributed_slice(riot_rs_rt::INIT_FUNCS)]
pub(crate) fn init() {
    riot_rs_rt::debug::println!("riot-rs-embassy::init()");
    let p = arch::OptionalPeripherals::from(arch::init(Default::default()));
    EXECUTOR.start(SWI);
    EXECUTOR.spawner().spawn(init_task(p)).unwrap();
    riot_rs_rt::debug::println!("riot-rs-embassy::init() done");
}

#[embassy_executor::task]
async fn init_task(peripherals: arch::OptionalPeripherals) {
    riot_rs_rt::debug::println!("riot-rs-embassy::init_task()");

    #[cfg(all(context = "nrf52", feature = "usb"))]
    {
        // nrf52840
        let clock: embassy_nrf::pac::CLOCK = unsafe { core::mem::transmute(()) };

        riot_rs_rt::debug::println!("nrf: enabling ext hfosc...");
        clock.tasks_hfclkstart.write(|w| unsafe { w.bits(1) });
        while clock.events_hfclkstarted.read().bits() != 1 {}
    }

    let init_args = InitializationArgs {
        peripherals: make_static!(Mutex::new(peripherals)),
    };

    let mut drivers = Drivers {
        #[cfg(feature = "usb_ethernet")]
        stack: make_static!(OnceCell::new()),
    };

    drivers.set_up_usb_ethernet(init_args).await;

    let spawner = Spawner::for_current_executor().await;

    for task in EMBASSY_TASKS {
        // FIXME: init all tasks before starting them
        match task(init_args.peripherals.lock().await.deref_mut(), init_args) {
            Ok(initialized_program) => initialized_program.start(spawner, drivers),
            Err(err) => panic!("Error while initializing a program: {err:?}"),
        }
    }

    riot_rs_rt::debug::println!("riot-rs-embassy::init_task() done");
}

pub trait UserProgram {
    fn initialize(
        peripherals: &mut embassy_nrf::OptionalPeripherals,
        init_args: InitializationArgs,
    ) -> Result<&dyn UserProgram, ProgramInitError>
    where
        Self: Sized;
    // TODO: make it so a user program cannot be started twice
    fn start(&self, spawner: embassy_executor::Spawner, drivers: Drivers); // TODO: or run?
}

#[derive(Debug)]
pub enum ProgramInitError {
    CannotObtainPeripheral,
}

/// Requires the [linkme] crate to be in scope.
#[macro_export]
macro_rules! riot_initialize {
    ($prog_type:ident) => {
        #[linkme::distributed_slice(riot_rs::embassy::EMBASSY_TASKS)]
        fn __init_program(
            peripherals: &mut embassy_nrf::OptionalPeripherals,
            init_args: InitializationArgs,
        ) -> Result<&dyn UserProgram, ProgramInitError> {
            $prog_type::initialize(peripherals, init_args)
        }
    };
}

#[cfg(feature = "usb_ethernet")]
impl Drivers {
    pub async fn set_up_usb_ethernet(&mut self, init_args: InitializationArgs) {
        let mut usb_builder = {
            let usb_config = usb_default_config();

            let usbd = init_args.peripherals.lock().await.USBD.take().unwrap();

            #[cfg(context = "nrf52")]
            let usb_driver = nrf52::usb::driver(usbd);

            #[cfg(context = "rp2040")]
            let usb_driver = rp2040::usb::driver(usbd);

            // Create embassy-usb DeviceBuilder using the driver and config.
            let builder = UsbBuilder::new(
                usb_driver,
                usb_config,
                &mut make_static!([0; 256])[..],
                &mut make_static!([0; 256])[..],
                &mut make_static!([0; 256])[..],
                &mut make_static!([0; 128])[..],
                &mut make_static!([0; 128])[..],
            );

            builder
        };

        // Our MAC addr.
        let our_mac_addr = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];

        let usb_cdc_ecm = {
            // Host's MAC addr. This is the MAC the host "thinks" its USB-to-ethernet adapter has.
            let host_mac_addr = [0x88, 0x88, 0x88, 0x88, 0x88, 0x88];

            use embassy_usb::class::cdc_ncm::{CdcNcmClass, State};

            // Create classes on the builder.
            CdcNcmClass::new(
                &mut usb_builder,
                make_static!(State::new()),
                host_mac_addr,
                64,
            )
        };

        let spawner = Spawner::for_current_executor().await;

        let usb_ethernet_device = {
            use embassy_usb::class::cdc_ncm::embassy_net::State as NetState;
            let (runner, device) = usb_cdc_ecm.into_embassy_net_device::<ETHERNET_MTU, 4, 4>(
                make_static!(NetState::new()),
                our_mac_addr,
            );

            spawner.spawn(usb_ncm_task(runner)).unwrap();

            device
        };

        let stack = {
            // Generate random seed
            // let mut rng = Rng::new(p.RNG, Irqs);
            // let mut seed = [0; 8];
            // rng.blocking_fill_bytes(&mut seed);
            // let seed = u64::from_le_bytes(seed);
            let seed = 1234u64;

            let config = network_config();

            // Init network stack
            let stack = &*make_static!(UsbEthernetStack::new(
                usb_ethernet_device,
                config,
                make_static!(StackResources::<2>::new()),
                seed
            ));

            spawner.spawn(usb_ethernet_task(stack)).unwrap();

            stack
        };

        let usb = usb_builder.build();
        spawner.spawn(usb_task(usb)).unwrap();

        // Do nothing if a stack is already initialized, as this should not happen anyway
        // TODO: should we panic instead?
        let _ = self.stack.set(stack);
    }
}

#[cfg(feature = "net")]
fn network_config() -> embassy_net::Config {
    #[cfg(not(feature = "override_network_config"))]
    {
        use embassy_net::{Ipv4Address, Ipv4Cidr, StaticConfigV4};
        embassy_net::Config::ipv4_static(StaticConfigV4 {
            address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
            dns_servers: heapless::Vec::new(),
            gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
        })
    }
    #[cfg(feature = "override_network_config")]
    {
        extern "Rust" {
            fn riot_rs_network_config() -> embassy_net::Config;
        }
        unsafe { riot_rs_network_config() }
    }
}

#[cfg(feature = "net")]
use embassy_net::tcp::TcpSocket;

#[cfg(feature = "net")]
impl Drivers {
    /// Returns a TCP socket.
    ///
    /// # Panics
    ///
    /// Panics if called before the network stack is initialized.
    pub fn get_tcp_socket<'a>(
        &'a self,
        rx_buffer: &'a mut [u8],
        tx_buffer: &'a mut [u8],
    ) -> TcpSocket {
        let stack = self.stack.get().unwrap();
        TcpSocket::new(stack, rx_buffer, tx_buffer)
    }
}
