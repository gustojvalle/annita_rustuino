use crate::{
    actuators::pump::set_pump_pressure,
    board::board::Board,
    functional::espresso_state::{push_snapshot, EspressoStateSnapshot},
    BOARD, ESPRESSO_SYSTEM_STACK,
};
use esp32_nimble::utilities::mutex::MutexGuard;
use once_cell::sync::OnceCell;
use std::{sync::Mutex, thread, time::Duration};

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
    flow_restriction: f32,
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
            flow_restriction: 2.0,
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
pub fn do_analog_espresso(config: &EspressoConfig, board: &mut Board) {
    println!("doing analog espresso");
    let mut button_state = true;
    while button_state {
        button_state = board.get_button_state();
        if !button_state {
            break;
        }
        println!("Making espresso");
        // Simulate making espresso
        println!("thread sleep");
        let espresso_snapshot = EspressoStateSnapshot::get_state(board).unwrap();
        push_snapshot(espresso_snapshot.clone());
        println!("get espresso_snapshot {:?}", espresso_snapshot);
        set_pump_pressure(
            &config._shot_config.pressure,
            &config._shot_config.flow_restriction,
            &espresso_snapshot,
        );
        std::thread::sleep(Duration::from_secs(1));
    }
}

pub fn do_auto_espresso(config: &EspressoConfig) {
    // TODO make espresso with the programable things.
}

pub fn do_auto_espresso_with_pressure_profile(config: EspressoConfig) {
    // TODO make espresso with the programable things.
}

pub fn do_espresso(board: &mut Board) {
    if let Some(config_lock) = ESPRESSO_CONFIG.get() {
        let config = config_lock.lock().unwrap();
        println!("config {:?}", config);
        match config.initialisation_type {
            InitialisationType::AnalogButton => {
                println!("doing analog espresso");
                do_analog_espresso(&config, board)
            }
            InitialisationType::Program => do_auto_espresso(&config),
        }
    } else {
        println!("EspressoConfig is not initialized");
    }
}
