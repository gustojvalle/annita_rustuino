use std::time::SystemTime;

use anyhow::Result;

use crate::{board::board::Board, sensors::temperature::read_temperature};

use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub enum MachineMode {
    ManualBrew,
    ShotProfiling,
    Steam,
    Descale,
}

#[derive(Debug, Serialize)]
pub struct MachineSnapshot {
    boiler_temp: f32,
    pump_state: f32,
    valve_state: bool,
    brew_button: bool,
    steam_button: bool,
    steam_button_on_time: Option<SystemTime>,
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

#[derive(Debug, Serialize)]
pub struct CoffeeMachineConfig {
    brew_temp_setpoint: u8,
    is_steam: bool,
    mode: MachineMode,
    machine_snapshot: MachineSnapshot,
}

impl CoffeeMachineConfig {
    pub fn default(board: &mut Board) -> CoffeeMachineConfig {
        CoffeeMachineConfig {
            brew_temp_setpoint: 90,
            is_steam: false,
            mode: MachineMode::ManualBrew,
            machine_snapshot: MachineSnapshot::get_machine_snapshot(board).unwrap(),
        }
    }
}
