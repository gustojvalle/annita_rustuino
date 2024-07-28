use anyhow::Result;
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::*;

use crate::board::board::Board;

fn convert_volt_to_pressure(adc_value: u16) -> f32 {
    let max_adc_value = 4095.0; // 12-bit ADC
    let initial_voltage = 0.5; // Reference voltage in volts, assuming 5V supply

    // Convert ADC value to voltage
    let adc_percentage = ((adc_value as f32) / max_adc_value) - (initial_voltage / 4.5);

    // Convert voltage to pressure (MPa)
    // let pressure_mpa = ((voltage) / 3.3) * 2.5;

    // Convert pressure from MPa to bar
    let pressure_bar = adc_percentage * 2.5 * 10.0;
    return pressure_bar;
}

pub fn read_pressure(board: &mut Board) -> Result<f32> {
    let mut adc = AdcDriver::new(&mut board.adc2)?;

    // configuring pin to analog read, you can regulate the adc input voltage range depending on your need
    // for this example we use the attenuation of 11db which sets the input voltage range to around 0-3.6V
    let config = AdcChannelConfig {
        attenuation: DB_11,
        calibration: true,
        ..Default::default()
    };
    let mut adc_pin = AdcChannelDriver::new(&mut adc, &mut board.gpio2, &config)?;
    let pressure = convert_volt_to_pressure(adc_pin.read()?);

    println!("ADC value: {}bar, {}", pressure, adc_pin.read()?);
    return Ok(pressure);
}
