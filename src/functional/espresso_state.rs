use std::{
    fmt,
    time::{Duration, SystemTime},
};

use crate::{
    board::board::Board,
    sensors::{
        flow::{self, calculate_espresso_flow},
        pressure::read_pressure,
        temperature,
    },
    ESPRESSO_SYSTEM_STACK,
};
use anyhow::Result;

pub struct EspressoStateSnapshot {
    pressure: f32,
    boiler_temp: f32,
    estimated_espresso_flow: f32,
    time: SystemTime,
    elapsed_time_from_last_read: Duration,
    estimated_weight: f32,
    measured_flow: flow::Flow,
    espresso_flow: f32,
}
impl fmt::Debug for EspressoStateSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EspressoStateSnapshot")
            .field("pressure", &self.pressure) // Assuming Modem doesn't implement Debug
            .field("boiler_temp", &self.boiler_temp) // Assuming Modem doesn't implement Debug
            .field("estimated_espresso_flow", &self.estimated_espresso_flow) // Assuming Modem doesn't implement Debug
            .field("time", &self.time) // Assuming Modem doesn't implement Debug
            .field(
                "elapsed_time_from_last_read",
                &self.elapsed_time_from_last_read,
            ) // Assuming Modem doesn't implement Debug
            .field("estimated_weight", &self.estimated_weight) // Assuming Modem doesn't implement Debug
            .field("measured_flow", &self.measured_flow) // Assuming Modem doesn't implement Debug
            .field("espresso_flow", &self.espresso_flow) // Assuming Modem doesn't implement Debug
            .finish()
    }
}

impl EspressoStateSnapshot {
    pub fn get_state(board: &mut Board) -> Result<EspressoStateSnapshot> {
        let pressure = match read_pressure(board) {
            Err(e) => {
                return anyhow::bail!(e);
            }
            Ok(temp) => temp,
        };
        let current_time = SystemTime::now();
        Ok(EspressoStateSnapshot {
            pressure: pressure,
            boiler_temp: temperature::read_temperature()?,
            estimated_espresso_flow: 0.0,
            estimated_weight: 0.0,
            measured_flow: flow::Flow {
                enter: 0.0,
                exit: 0.0,
            },
            time: current_time,
            elapsed_time_from_last_read: calculate_elapsed_time_from_last_snapshot(current_time)?,
            espresso_flow: calculate_espresso_flow()?,
        })
    }
}
pub fn push_snapshot(snapshot: EspressoStateSnapshot) {
    if let Some(stack) = ESPRESSO_SYSTEM_STACK.get() {
        let mut stack = stack.lock().expect("Failed to acquire lock");
        stack.push(snapshot);
    } else {
        eprintln!("ESPRESSO_SYSTEM_STACK is not initialized");
    }
}

fn calculate_elapsed_time_from_last_snapshot(current_time: SystemTime) -> Result<Duration> {
    if let Some(stack) = ESPRESSO_SYSTEM_STACK.get() {
        let stack = stack.lock().expect("Failed to acquire lock");
        return Ok(stack[stack.len() - 1]
            .time
            .duration_since(current_time)
            .unwrap());
    } else {
        unreachable!("failed to get stack");
    }
}
fn calculated_weight() {
    // TODO calculate the weight with the flow in and flow out.
}
