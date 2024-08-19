use std::{
    error::Error,
    fmt,
    time::{Duration, SystemTime},
};

use crate::{
    actuators::{psm::calculate_cps, pump::get_pump_flow},
    board::board::Board,
    sensors::{
        flow::{self, calculate_espresso_flow},
        pressure::read_pressure,
        temperature,
    },
    ESPRESSO_SYSTEM_STACK,
};
use anyhow::Result;
use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct EspressoStateSnapshot {
    pub pressure: f32,
    pub boiler_temp: f32,
    pub estimated_espresso_flow: f32,
    pub time: SystemTime,
    pub elapsed_time_from_last_read: Duration,
    pub estimated_weight: f32,
    pub measured_flow: flow::Flow,
    pub espresso_flow: f32,
    pub pressure_change_speed: f32,
    pub pump_flow: f32,
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
            .field("pressure_change_speed", &self.pressure_change_speed) // Assuming Modem doesn't implement Debug
            .field("pump_flow", &self.pump_flow) // Assuming Modem doesn't implement Debug
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
        let cps = calculate_cps();
        let elapsed_time = match calculate_elapsed_time_from_last_snapshot(current_time) {
            Ok(time) => time,
            Err(err) => Duration::new(0, 0),
        };
        let espresso_snapshot = EspressoStateSnapshot {
            pressure: pressure,
            boiler_temp: temperature::read_temperature()?,
            estimated_espresso_flow: 0.0,
            estimated_weight: 0.0,
            measured_flow: flow::Flow {
                enter: 0.0,
                exit: 0.0,
            },
            time: current_time,
            elapsed_time_from_last_read: elapsed_time,
            espresso_flow: calculate_espresso_flow()?,
            // TODO calculate pressure change speed
            pressure_change_speed: 0.0,
            pump_flow: get_pump_flow(cps, &pressure),
        };
        Ok(espresso_snapshot)
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

pub fn pop_snapshot() {
    if let Some(stack) = ESPRESSO_SYSTEM_STACK.get() {
        let mut stack = stack.lock().expect("Failed to acquire lock");
        stack.pop();
    } else {
        eprintln!("ESPRESSO_SYSTEM_STACK is not initialized");
    }
}

fn calculate_elapsed_time_from_last_snapshot(current_time: SystemTime) -> Result<Duration> {
    if let Some(stack) = ESPRESSO_SYSTEM_STACK.get() {
        let stack = stack.lock().expect("Failed to acquire lock");
        if stack.len() == 0 {
            return Ok(Duration::new(0, 0));
        }
        return Ok(current_time
            .duration_since(stack[stack.len() - 1].time)
            .unwrap());
    } else {
        unreachable!("failed to get stack")
    }
}
fn calculate_presssure_change_speed(
    current_pressure: f32,
    current_time: SystemTime,
) -> Result<f32> {
    if let Some(stack) = ESPRESSO_SYSTEM_STACK.get() {
        let stack = stack.lock().expect("Failed to acquire lock");
        let duration = stack[stack.len() - 1]
            .time
            .duration_since(current_time)
            .unwrap();
        return Ok((current_pressure - stack[stack.len() - 1].pressure) / duration.as_secs_f32());
    } else {
        unreachable!("failed to get stack");
    }
}

fn calculated_weight() {
    // TODO calculate the weight with the flow in and flow out.
}
