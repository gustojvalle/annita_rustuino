use anyhow::{bail, Result};
use board::board::Board;
use coffee_machine::config::CoffeeMachineConfig;
use esp32_nimble::{uuid128, NimbleProperties};
use esp_idf_hal::gpio::Gpio2;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_sys as _;
use functional::espresso_state::EspressoStateSnapshot;
use sensors::temperature;
use serde::Deserialize;
use serde::Serialize;
// use functional::espresso::EspressoConfig;

#[derive(Deserialize, Serialize)]
struct Res {
    led: Option<bool>,
    temperature: u8,
}
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

mod sensors {
    pub mod flow;
    pub mod pressure;
    pub mod temperature;
}

mod board {
    pub mod board;
}

mod coffee_machine {
    pub mod config;
}

mod connectivity {
    pub mod bt;
    // pub mod wifi;
}

mod functional {
    pub mod espresso;
    pub mod espresso_state;
}

mod actuators {
    pub mod boiler;
    pub mod psm;
    pub mod pump;
}

use log::info;
use once_cell::sync::OnceCell;

static BOARD: OnceCell<Arc<Mutex<Board<'static>>>> = OnceCell::new();
static ESPRESSO_SYSTEM_STACK: OnceCell<
    Mutex<Vec<functional::espresso_state::EspressoStateSnapshot>>,
> = OnceCell::new();
static MACHINE_CONFIG: OnceCell<Arc<Mutex<CoffeeMachineConfig>>> = OnceCell::new();

pub fn init_board() {
    let board = Board::<'static>::init().unwrap();
    BOARD.set(Arc::new(Mutex::new(board))).unwrap();
}

pub fn init_espresso_memory_stack() {
    ESPRESSO_SYSTEM_STACK.set(Mutex::new(Vec::new())).unwrap();
}

pub fn init_machine_config(board: &mut Board) {
    let machine_configuration = CoffeeMachineConfig::default(board);
    MACHINE_CONFIG
        .set(Arc::new(Mutex::new(machine_configuration)))
        .unwrap();
}

fn init_bluetooth(
    board: &mut Board,
    onboard_led:&mut Arc<Mutex<PinDriver<'static, Gpio2, esp_idf_hal::gpio::Output>>>,
) {
    let snapshot_service =
        board.set_ble_service(uuid128!("02550882-1f94-4b1e-a448-e2f7691ac386"), "snapshot");
    board.set_ble_characteristic(
        snapshot_service,
        uuid128!("799f64c7-362b-44b9-9413-2d2ee8e5a784"),
        "snapshot_state",
        NimbleProperties::READ | NimbleProperties::NOTIFY | NimbleProperties::INDICATE,
        b"initializing snapshot no measures yet",
    );

    let config_service = board.set_ble_service(
        uuid128!("497b30e0-c4be-4bca-8a38-cc74e84cd4ce"),
        "configuration",
    );
    board.set_ble_characteristic(
        config_service.clone(),
        uuid128!("e0caff18-2598-4bac-86f7-50b06c3478a6"),
        "shot_config",
        NimbleProperties::READ
            | NimbleProperties::NOTIFY
            | NimbleProperties::INDICATE
            | NimbleProperties::WRITE,
        b"initializing configuration",
    );
    let machine_config_publisher = board.set_ble_characteristic(
        config_service,
        uuid128!("35124b97-6292-4c46-ae49-21171df21527"),
        "machine_configuration",
        NimbleProperties::WRITE
            | NimbleProperties::READ
            | NimbleProperties::NOTIFY
            | NimbleProperties::INDICATE,
        b"initializing configuration",
    );

    let onboard_led_clone = onboard_led.clone();
    machine_config_publisher.lock().on_write(move |val| {
        let mut onboard_led = onboard_led_clone.lock().unwrap();
        log::info!(
            "Configuration recv_data: {:?}, current_data: {:?}",
            std::str::from_utf8(&mut val.recv_data()),
            std::str::from_utf8(&mut val.current_data())
        );


        // Try to parse the JSON data
        let json: Result<Res, serde_json::Error> = serde_json::from_slice(val.recv_data());
        match json {
            Ok(parsed_data) => {
                let led_level = if parsed_data.led.unwrap() {
                    esp_idf_hal::gpio::Level::Low
                } else {
                    esp_idf_hal::gpio::Level::High
                };

                if let Err(e) = onboard_led.set_level(led_level) {
                    log::error!("Failed to set LED level: {:?}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to deserialize JSON: {:?}", e);
            }
        }
    });

    // machine_config_publisher.lock().on_write(move |val| {
    //     // let mut new_board = new_board_mutex.lock().unwrap();
    //     log::info!(
    //         "heres the configuration recv_data: {:?}, current_data: {:?}",
    //         std::str::from_utf8(val.recv_data()),
    //         std::str::from_utf8(val.current_data())
    //     );
    //     let rec_str = std::str::from_utf8(val.recv_data()).unwrap();
    //     let json: Res = serde_json::from_str(&rec_str).unwrap();
    //
    //     // if json.led.unwrap() {
    //     //     new_board
    //     //         .onboard_led
    //     //         .set_level(esp_idf_hal::gpio::Level::Low)
    //     //         .unwrap();
    //     // } else {
    //     //     new_board
    //     //         .onboard_led
    //     //         .set_level(esp_idf_hal::gpio::Level::High)
    //     //         .unwrap();
    //     // };
    // });

    match board.ble_device.get_advertising().lock().start() {
        Ok(_) => log::info!("Succesful ble init"),
        Err(err) => log::info!("Failed to init ble with error: {:?}", err),
    };
}

fn main() -> Result<()> {
    let sysloop = EspSystemEventLoop::take().unwrap();

    // Configure Advertiser Data
    thread::sleep(Duration::from_secs(5));
    init_espresso_memory_stack();
    init_board();

    // Link patches required for ESP-IDF
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");
    log::info!("Connecting to WiFi");

    // let sysloop_clone = sysloop.clone(); // Clone the event loop

    let board_clone_main = BOARD.get().expect("BOARD not initialized").clone();

    functional::espresso::init_espresso_config();
    log::info!("Connected, starting loop");

    let mut board_main = board_clone_main.lock().unwrap();

    init_machine_config(&mut board_main);
    let machine_config = MACHINE_CONFIG
        .get()
        .expect("MACHINE_CONFIG not initialized")
        .clone();

    //initializing maching configuration.
    let machine_lock = machine_config.lock().unwrap();
    let machine_config_string = serde_json::to_string(&*machine_lock).unwrap();
    let mut onboard_led = board_main.onboard_led.clone();

    init_bluetooth(&mut board_main,&mut onboard_led);

    board_main
        .ble_characteristics
        .get("machine_configuration")
        .unwrap()
        .lock()
        .set_value(machine_config_string.as_bytes())
        .notify();

    // Main loop
    log::info!("Connected, starting loop");

    loop {
        //
        // let pressure = match sensors::pressure::read_pressure(&mut *board_main) {
        //     Ok(pressure) => pressure,
        //     Err(_) => 0.0,
        // };

        // let espresso_snapshot = EspressoStateSnapshot::get_state(&mut board_main).unwrap();
        // functional::espresso_state::push_snapshot(espresso_snapshot.clone());
        //
        // let json_data = serde_json::to_string(&espresso_snapshot).unwrap();
        //
        // let json_bytes = json_data.as_bytes();
        //
        // board_main
        //     .ble_characteristics
        //     .get("snapshot_state")
        //     .unwrap()
        //     .lock()
        //     .set_value(&json_bytes)
        //     .notify();

        // // Borrow button state immutably
        let button_state = board_main.get_button_state();
        if button_state {
            functional::espresso::do_espresso(&mut *board_main);
        }

        if let Some(stack) = ESPRESSO_SYSTEM_STACK.get() {
            let mut stack = stack.lock().expect("Failed to acquire lock");
            if stack.len() > 2 {
                stack.pop();
                // eprintln!("Poped out from stack");
            }
        } else {
            eprintln!("ESPRESSO_SYSTEM_STACK is not initialized");
        }
        // start_espresso_thread(board_clone_main.clone());
        std::thread::sleep(Duration::from_millis(500));
    }
}
