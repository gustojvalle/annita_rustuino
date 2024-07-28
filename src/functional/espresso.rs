use crate::{
    functional::espresso_state::{push_snapshot, EspressoStateSnapshot},
    BOARD, ESPRESSO_SYSTEM_STACK,
};
use once_cell::sync::OnceCell;
use std::sync::Mutex;

#[derive(Debug, PartialEq)]
pub enum EspressoType {
    Double,
    Lungo,
    Single,
    Ristretto,
}

#[derive(Debug, PartialEq)]
pub enum InitialisationType {
    AnalogButton,
    Program,
}

#[derive(Debug)]
pub struct ShotConfig {
    grains_weight_in: f32,
    espresso_yield: Option<f32>,
    override_final_weight: Option<f32>,
    override_shot_time: Option<u32>,
    pressure: f32,
    // TODO create pressure profile and flow profile for the espresso machine.
    // pressure_profile
}
impl Default for ShotConfig {
    fn default() -> Self {
        ShotConfig {
            grains_weight_in: 16.0,
            espresso_yield: Some(2.0),
            pressure: 9.0,
            override_final_weight: None,
            override_shot_time: None,
        }
    }
}

#[derive(Debug)]
pub struct EspressoConfig {
    pub initialisation_type: InitialisationType,
    pub _shot_config: ShotConfig,
}

pub fn init_espresso_config() {
    let config = EspressoConfig {
        initialisation_type: InitialisationType::AnalogButton,
        _shot_config: ShotConfig::default(),
    };

    ESPRESSO_CONFIG.set(Mutex::new(config)).unwrap();
}
static ESPRESSO_CONFIG: OnceCell<Mutex<EspressoConfig>> = OnceCell::new();
pub fn do_analog_espresso() {
    while let Some(board_lock) = BOARD.get() {
        let mut board = board_lock.lock().unwrap();
        if board.get_button_state() {
            println!("Making espresso");
            // Simulate making espresso
            std::thread::sleep(std::time::Duration::from_secs(1));
            let espresso_snapshot = EspressoStateSnapshot::get_state(&mut board).unwrap();
            push_snapshot(espresso_snapshot)
        } else {
            break;
        }
    }
}

pub fn do_auto_espresso(config: &EspressoConfig) {
    // TODO make espresso with the programable things.
}

pub fn do_auto_espresso_with_pressure_profile(config: EspressoConfig) {
    // TODO make espresso with the programable things.
}

pub fn do_espresso() {
    if let Some(config_lock) = ESPRESSO_CONFIG.get() {
        let config = config_lock.lock().unwrap();
        match config.initialisation_type {
            InitialisationType::AnalogButton => do_analog_espresso(),
            InitialisationType::Program => do_auto_espresso(&config),
        }
    } else {
        println!("EspressoConfig is not initialized");
    }
}
