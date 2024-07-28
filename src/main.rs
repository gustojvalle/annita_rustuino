use anyhow::bail;
use anyhow::Result;
use board::board::Board;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_sys as _;

mod sensors;
use functional::espresso::init_espresso_config;
use functional::espresso_state;
use functional::espresso_state::EspressoStateSnapshot;
use sensors::{flow, pressure, temperature}; // Using the submodules
mod board {
    pub mod board;
}

mod coffee_machine {
    pub mod config;
}

mod connectivity {
    pub mod connectivity;
}
mod functional {
    pub mod espresso;
    pub mod espresso_state;
}
mod actuators {
    pub mod boiler;
}

use log::info;
use once_cell::sync::OnceCell;
use std::sync::Mutex;

static BOARD: OnceCell<Mutex<Board>> = OnceCell::new();

static ESPRESSO_SYSTEM_STACK: OnceCell<Mutex<Vec<EspressoStateSnapshot>>> = OnceCell::new();

pub fn init_board(peripherals: Peripherals) {
    let board = Board::init(peripherals);
    BOARD.set(Mutex::new(board)).unwrap();
}

pub fn init_espresso_memory_stack() {
    ESPRESSO_SYSTEM_STACK.set(Mutex::new(Vec::new())).unwrap();
}

fn main() -> Result<()> {
    let sysloop = EspSystemEventLoop::take().unwrap();
    // It is necessary to call this function once. Otherwise some patches to the runtime
    let peripherals = Peripherals::take().unwrap();
    init_espresso_memory_stack();

    init_board(peripherals);
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    // let mut board = board::board::Board::init(peripherals);

    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");
    log::info!("connecting to wifi");

    init_espresso_config();
    // let _wifi = match wifi("Pixel6", "UgAGpzgcjRdg", peripherals.modem, sysloop) {
    // let _wifi = match connectivity::connectivity::wifi("Pixel6", "1234567890", board.modem, sysloop)
    // {
    //     Ok(inner) => inner,
    //     Err(err) => bail!("failed to connect {:?}", err),
    // };

    log::info!("Connected starting loop");
    loop {
        if let Some(board_lock) = BOARD.get() {
            let mut board = board_lock.lock().unwrap();

            if let Err(e) = sensors::pressure::read_pressure(&mut board) {
                eprintln!("Failed to read pressure: {:?}", e);
            }

            // Borrow button state immutably
            if board.get_button_state() {
                functional::espresso::do_espresso();
            }
        }

        info!("Hello, world!");

        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
