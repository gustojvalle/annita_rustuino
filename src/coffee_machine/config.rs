use std::time::SystemTime;

use anyhow::Result;

use crate::{board::board::Board, sensors::temperature::read_temperature};

#[derive(Debug, PartialEq)]
pub enum MachineMode {
    ManualBrew,
    ShotProfiling,
    Steam,
    Descale,
}

#[derive(Debug)]
pub struct MachineSnapshot {
    boiler_temp: f32,
    pump_state: f32,
    valve_state: bool,
    brew_button: bool,
    steam_button: bool,
    steam_button_on_time: Option<SystemTime>,
}

struct CoffeeMachineConfig {
    brew_temp_setpoint: f32,
    mode: MachineMode,
    machine_snapshot: MachineSnapshot,
}

impl MachineSnapshot {
    fn get_machine_snapshot(board: &mut Board) -> Result<MachineSnapshot> {
        Ok(MachineSnapshot {
            boiler_temp: read_temperature()?,
            // TODO set the pump_state reading,
            pump_state: 0.0,
            // TODO set the  valve state reading,
            valve_state: false,
            // TODO set the  brew state reading,
            brew_button: false,
            // TODO set the  steam state reading,
            steam_button: false,
            // TODO set the  steam button on time reading,
            steam_button_on_time: None,
        })
    }
}
