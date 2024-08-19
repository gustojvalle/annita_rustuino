use crate::functional::espresso_state::EspressoStateSnapshot;

pub const PRESSURE_INEFFICIENCY_COEFFICIENT: [f32; 7] =
    [0.045, 0.015, 0.0033, 0.000685, 0.000045, 0.009, -0.0018];

pub static FLOW_PER_CLICK_AT_ZERO_BAR: f32 = 0.27;
pub static MAX_PUMP_CLICKS_PER_SECOND: i32 = 50;
pub static FPC_MULTIPLIER: f32 = 1.2;
pub static PUMP_RANGE: u8 = 100;

struct PumpState {
    clicks: u16,
}

pub struct PumpConfig {
    flow_per_click_at_zero_bar: f32,
    max_pump_clicks_per_second: i32,
    fpc_multiplier: f32,
}
impl PumpConfig {
    pub fn default() -> PumpConfig {
        return PumpConfig {
            flow_per_click_at_zero_bar: FLOW_PER_CLICK_AT_ZERO_BAR,
            max_pump_clicks_per_second: MAX_PUMP_CLICKS_PER_SECOND,
            fpc_multiplier: FPC_MULTIPLIER,
        };
    }
}

// constants pulle from gaggiuino assuming the pump will be similar.
// https://github.com/Zer0-bit/gaggiuino/blob/release/stm32-blackpill/src/peripherals/pump.cpp

pub fn pump_init(power_line_frequency: i32, pump_flow_at_zero: f32) -> PumpConfig {
    return PumpConfig::default();
}

fn get_pump_pct(
    target_pressure: &f32,
    flow_restriction: &f32,
    current_state: &EspressoStateSnapshot,
) -> f32 {
    if target_pressure == &0.0 {
        return 0.0;
    }

    let diff = target_pressure - current_state.pressure;
    let max_pump_pct = if flow_restriction <= &0.0 {
        1.0
    } else {
        get_clicks_per_second_for_flow(flow_restriction, &current_state.pressure)
            / MAX_PUMP_CLICKS_PER_SECOND as f32
    };
    let pump_pct_to_maintain_flow =
        get_clicks_per_second_for_flow(&current_state.pump_flow, &current_state.pressure)
            / MAX_PUMP_CLICKS_PER_SECOND as f32;

    if diff > 2.0 {
        return max_pump_pct.min(0.25 + 0.2 * diff);
    }

    if diff > 0.0 {
        return max_pump_pct.min(pump_pct_to_maintain_flow * 0.95 + 0.1 + 0.2 * diff);
    }

    if current_state.pressure_change_speed < 0.0 {
        return max_pump_pct.min(pump_pct_to_maintain_flow * 0.2);
    }

    0.0
}

pub fn set_pump_pressure(
    target_pressure: &f32,
    flow_restriction: &f32,
    current_state: &EspressoStateSnapshot,
) {
    let pump_pct = get_pump_pct(target_pressure, flow_restriction, current_state);
    set_pump_to_raw_value((pump_pct * PUMP_RANGE as f32) as u8);
}

fn set_pump_off() {
    pump_set(0);
}

fn set_pump_full_on() {
    pump_set(PUMP_RANGE);
}

fn set_pump_to_raw_value(val: u8) {
    pump_set(val);
}

fn get_pump_flow_per_click(pressure: &f32) -> f32 {
    let fpc = (PRESSURE_INEFFICIENCY_COEFFICIENT[5] / pressure
        + PRESSURE_INEFFICIENCY_COEFFICIENT[6])
        * (-pressure * pressure)
        + (FLOW_PER_CLICK_AT_ZERO_BAR - PRESSURE_INEFFICIENCY_COEFFICIENT[0])
        - (PRESSURE_INEFFICIENCY_COEFFICIENT[1]
            + (PRESSURE_INEFFICIENCY_COEFFICIENT[2]
                - (PRESSURE_INEFFICIENCY_COEFFICIENT[3]
                    - PRESSURE_INEFFICIENCY_COEFFICIENT[4] * pressure)
                    * pressure)
                * pressure)
            * pressure;
    fpc * FPC_MULTIPLIER
}

pub fn get_pump_flow(cps: i32, pressure: &f32) -> f32 {
    (cps as f32) * get_pump_flow_per_click(pressure)
}

fn get_clicks_per_second_for_flow(flow: &f32, pressure: &f32) -> f32 {
    if flow == &0.0 {
        return 0.0;
    }
    let flow_per_click = get_pump_flow_per_click(pressure);
    let cps = flow / flow_per_click;
    cps.min(MAX_PUMP_CLICKS_PER_SECOND as f32)
}

fn set_pump_flow(
    target_flow: &f32,
    pressure_restriction: &f32,
    current_state: &EspressoStateSnapshot,
) {
    if pressure_restriction > &0.0 && current_state.pressure > pressure_restriction * &0.5 {
        set_pump_pressure(pressure_restriction, target_flow, current_state);
    } else {
        let pump_pct = get_clicks_per_second_for_flow(target_flow, &current_state.pressure)
            / MAX_PUMP_CLICKS_PER_SECOND as f32;
        set_pump_to_raw_value((pump_pct * PUMP_RANGE as f32) as u8);
    }
}

// Paceholder functions for pump control, watchdog, etc.
fn pump_set(val: u8) {
    // Set the pump to the given raw value
    rbd_dimmer::set_power(0, val).unwrap();
}

// Placeholder for TIM9, assuming a constant value or variable
const TIM9: u32 = 9;
