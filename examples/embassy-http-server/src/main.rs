#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

mod pins;
mod routes;

use riot_rs as _;

use riot_rs::embassy::{network, Spawner};
use riot_rs::rt::debug::println;

use embassy_net::tcp::TcpSocket;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Duration;
use picoserve::routing::get;
use static_cell::make_static;

#[cfg(feature = "button-readings")]
use embassy_nrf::gpio::{Input, Pin, Pull};

#[cfg(feature = "leds")]
use embassy_nrf::gpio::{Level, Output, OutputDrive};

struct AppState {
    #[cfg(feature = "button-readings")]
    buttons: ButtonInputs,
}

#[cfg(feature = "button-readings")]
#[derive(Copy, Clone)]
struct ButtonInputs(&'static Mutex<CriticalSectionRawMutex, Buttons>);

#[cfg(all(feature = "button-readings", builder = "nrf52840dk"))]
const BUTTON_COUNT: usize = 4;
#[cfg(feature = "button-readings")]
struct Buttons([Input<'static>; BUTTON_COUNT]);

#[cfg(all(feature = "leds", builder = "nrf52840dk"))]
const LED_COUNT: usize = 4;
#[cfg(feature = "leds")]
struct Leds([Output<'static>; LED_COUNT]);

#[cfg(feature = "button-readings")]
impl picoserve::extract::FromRef<AppState> for ButtonInputs {
    fn from_ref(state: &AppState) -> Self {
        state.buttons
    }
}

type AppRouter = impl picoserve::routing::PathRouter<AppState>;

const WEB_TASK_POOL_SIZE: usize = 2;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    id: usize,
    app: &'static picoserve::Router<AppRouter, AppState>,
    config: &'static picoserve::Config<Duration>,
    state: AppState,
) -> ! {
    let stack = network::network_stack().await.unwrap();

    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        println!("{}: Listening on TCP:80...", id);
        if let Err(e) = socket.accept(80).await {
            println!("{}: accept error: {:?}", id, e);
            continue;
        }

        let remote_endpoint = socket.remote_endpoint();

        println!("{}: Received connection from {:?}", id, remote_endpoint);

        match picoserve::serve_with_state(app, config, &mut [0; 2048], socket, &state).await {
            Ok(handled_requests_count) => {
                println!(
                    "{} requests handled from {:?}",
                    handled_requests_count, remote_endpoint,
                );
            }
            Err(err) => println!("{:?}", err),
        }
    }
}

#[riot_rs::main]
async fn main(
    #[cfg(feature = "button-readings")] buttons: pins::Buttons,
    #[cfg(feature = "leds")] leds: pins::Leds,
) {
    let spawner = Spawner::for_current_executor().await;

    #[cfg(feature = "button-readings")]
    let buttons = {
        let button_inputs = {
            let buttons = Buttons([
                Input::new(buttons.btn1.degrade(), Pull::Up),
                Input::new(buttons.btn2.degrade(), Pull::Up),
                Input::new(buttons.btn3.degrade(), Pull::Up),
                Input::new(buttons.btn4.degrade(), Pull::Up),
            ]);
            let buttons = make_static!(Mutex::new(buttons));

            #[cfg(feature = "leds")]
            let leds = {
                Leds([
                    Output::new(leds.led1.degrade(), Level::Low, OutputDrive::Standard),
                    Output::new(leds.led2.degrade(), Level::Low, OutputDrive::Standard),
                    Output::new(leds.led3.degrade(), Level::Low, OutputDrive::Standard),
                    Output::new(leds.led4.degrade(), Level::Low, OutputDrive::Standard),
                ])
            };

            #[cfg(feature = "leds-sync-buttons")]
            spawner.spawn(led_task(leds, buttons)).unwrap();

            ButtonInputs(buttons)
        };

        button_inputs
    };

    fn make_app() -> picoserve::Router<AppRouter, AppState> {
        let router = picoserve::Router::new().route("/", get(routes::index));
        #[cfg(feature = "button-readings")]
        let router = router.route("/buttons", get(routes::buttons));
        router
    }

    let app = make_static!(make_app());

    let config = make_static!(picoserve::Config::new(picoserve::Timeouts {
        start_read_request: Some(Duration::from_secs(5)),
        read_request: Some(Duration::from_secs(1)),
        write: Some(Duration::from_secs(1)),
    }));

    for id in 0..WEB_TASK_POOL_SIZE {
        let app_state = AppState {
            #[cfg(feature = "button-readings")]
            buttons,
        };
        spawner.spawn(web_task(id, app, config, app_state)).unwrap();
    }
}

#[cfg(feature = "leds-sync-buttons")]
#[embassy_executor::task]
async fn led_task(mut leds: Leds, buttons: &'static Mutex<CriticalSectionRawMutex, Buttons>) {
    use embassy_time::Timer;

    loop {
        for (button, led) in buttons.lock().await.0.iter().zip(leds.0.iter_mut()) {
            if button.is_low() {
                led.set_low();
            } else {
                led.set_high();
            }
        }

        // Avoid keeping the mutex locked all the time
        Timer::after(Duration::from_millis(50)).await;
    }
}

#[no_mangle]
fn riot_rs_network_config() -> embassy_net::Config {
    use embassy_net::Ipv4Address;

    embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
        address: embassy_net::Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 61), 24),
        dns_servers: heapless::Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    })
}
