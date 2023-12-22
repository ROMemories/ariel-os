#![no_main]
#![no_std]
#![feature(type_alias_impl_trait)]
#![feature(used_with_arg)]

use riot_rs::{
    self as _,
    embassy::{riot_initialize, ProgramInitError, UserProgram},
};

use riot_rs::embassy::{Drivers, InitializationArgs};
use riot_rs::rt::debug::println;

use embassy_nrf::gpio::{AnyPin, Input, Pin, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Duration;
use picoserve::{
    extract::State,
    response::{DebugValue, IntoResponse, Json},
    routing::{get, parse_path_segment},
};
use static_cell::make_static;

struct EmbassyTimer;

impl picoserve::Timer for EmbassyTimer {
    type Duration = embassy_time::Duration;
    type TimeoutError = embassy_time::TimeoutError;

    async fn run_with_timeout<F: core::future::Future>(
        &mut self,
        duration: Self::Duration,
        future: F,
    ) -> Result<F::Output, Self::TimeoutError> {
        embassy_time::with_timeout(duration, future).await
    }
}

struct AppState {
    button: ButtonInput,
}

#[derive(Copy, Clone)]
struct ButtonInput(&'static Mutex<CriticalSectionRawMutex, Input<'static, AnyPin>>);

impl ButtonInput {
    pub async fn is_high(&self) -> bool {
        self.0.lock().await.is_high()
    }

    pub async fn is_low(&self) -> bool {
        self.0.lock().await.is_low()
    }
}

impl picoserve::extract::FromRef<AppState> for ButtonInput {
    fn from_ref(state: &AppState) -> Self {
        state.button
    }
}

type AppRouter = impl picoserve::routing::PathRouter<AppState>;

const WEB_TASK_POOL_SIZE: usize = 2;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    drivers: Drivers,
    id: usize,
    app: &'static picoserve::Router<AppRouter, AppState>,
    config: &'static picoserve::Config<Duration>,
    state: AppState,
) -> ! {
    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];

    loop {
        let mut socket = drivers.get_tcp_socket(&mut rx_buffer, &mut tx_buffer);

        println!("{}: Listening on TCP:80...", id);
        if let Err(e) = socket.accept(80).await {
            println!("{}: accept error: {:?}", id, e);
            continue;
        }

        println!(
            "{}: Received connection from {:?}",
            id,
            socket.remote_endpoint()
        );

        let (socket_rx, socket_tx) = socket.split();

        match picoserve::serve_with_state(
            app,
            EmbassyTimer,
            config,
            &mut [0; 2048],
            socket_rx,
            socket_tx,
            &state,
        )
        .await
        {
            Ok(handled_requests_count) => {
                println!(
                    "{} requests handled from {:?}",
                    handled_requests_count,
                    socket.remote_endpoint()
                );
            }
            Err(err) => println!("{:?}", err),
        }
    }
}

async fn index() -> impl IntoResponse {
    picoserve::response::File::html(include_str!("../static/index.html"))
}

async fn read_button(State(button): State<ButtonInput>) -> impl IntoResponse {
    #[derive(serde::Serialize)]
    struct Buttons {
        button1: bool,
    }

    let button_pressed = button.is_low().await;
    if button_pressed {
        Json(Buttons { button1: true })
    } else {
        Json(Buttons { button1: false })
    }
}

#[no_mangle]
fn riot_rs_network_config() -> embassy_net::Config {
    use embassy_net::{Ipv4Address, Ipv4Cidr, StaticConfigV4};
    embassy_net::Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(10, 42, 0, 62), 24),
        dns_servers: heapless::Vec::new(),
        gateway: Some(Ipv4Address::new(10, 42, 0, 1)),
    })
}

struct HttpServer {
    app: &'static picoserve::Router<AppRouter, AppState>,
    config: &'static picoserve::Config<Duration>,
    button_input: ButtonInput,
}

impl UserProgram for HttpServer {
    fn initialize(
        peripherals: &mut embassy_nrf::OptionalPeripherals,
        init_args: InitializationArgs,
    ) -> Result<&dyn UserProgram, ProgramInitError> {
        fn make_app() -> picoserve::Router<AppRouter, AppState> {
            picoserve::Router::new()
                .route("/", get(index))
                .route("/button", get(read_button))
        }

        let app = make_static!(make_app());

        let config = make_static!(picoserve::Config {
            start_read_request_timeout: Some(Duration::from_secs(5)),
            read_request_timeout: Some(Duration::from_secs(1)),
            write_timeout: Some(Duration::from_secs(1)),
        });

        let button_pin = peripherals
            .P0_11
            .take()
            .ok_or(ProgramInitError::CannotObtainPeripheral)?;
        let button = Input::new(button_pin.degrade(), Pull::Up);

        let button_input = ButtonInput(make_static!(Mutex::new(button)));

        Ok(make_static!(HttpServer {
            button_input,
            app,
            config
        }))
    }

    fn start(&self, spawner: embassy_executor::Spawner, drivers: Drivers) {
        for id in 0..WEB_TASK_POOL_SIZE {
            let app_state = AppState {
                button: self.button_input,
            };
            spawner
                .spawn(web_task(drivers, id, self.app, self.config, app_state))
                .unwrap();
        }
    }
}

// TODO: this could be replaced by an attribute proc-macro on HttpServer::initialize()
riot_initialize!(HttpServer);

#[no_mangle]
fn riot_main() {
    println!(
        "Hello from riot_main()! Running on a {} board.",
        riot_rs::buildinfo::BOARD
    );

    loop {}
}
