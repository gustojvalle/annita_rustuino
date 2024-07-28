use core::fmt;
use esp_idf_hal::adc::ADC2;
use esp_idf_hal::gpio::{
    Gpio10, Gpio2, Gpio4, Gpio5, Gpio6, Gpio7, Gpio8, Gpio9, Input, Output, PinDriver, Pull,
};
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_sys::EspError;

#[derive(Debug, PartialEq)]
pub enum BoillerState {
    On,
    Off,
}
pub struct Board<'a> {
    pub modem: Modem,
    pub adc2: ADC2,
    pub pressure_pin: Gpio2,
    pub button_state: bool,
    pub pump: Gpio4,
    pub boiller: PinDriver<'a, Gpio5, Output>,
    pub three_way_valve: Gpio6,
    pub temperature_sensor_one: Gpio7,
    pub temperature_sensor_two: Gpio8,
    pub temperature_sensor_three: Gpio10,
    pub boiller_state: BoillerState,
    button: PinDriver<'a, Gpio9, Input>,
}
impl<'a> Board<'a> {
    pub fn init(p: Peripherals) -> Board<'a> {
        let adc2 = p.adc2;
        let modem = p.modem;
        let pressure_pin: Gpio2 = p.pins.gpio2;
        let button_pin = p.pins.gpio9;
        let pump = p.pins.gpio4;
        let boiller_pin = p.pins.gpio5;
        let three_way_valve = p.pins.gpio6;
        let temperature_sensor_one = p.pins.gpio7;
        let temperature_sensor_two = p.pins.gpio8;
        let temperature_sensor_three = p.pins.gpio10;
        let mut button = PinDriver::input(button_pin).unwrap();
        button.set_pull(Pull::Down).unwrap();
        let button_state = button.is_high();
        let mut boiller = PinDriver::output(boiller_pin).unwrap();
        let boiller_state = match boiller.set_high() {
            Ok(()) => BoillerState::On,
            Err(_) => BoillerState::Off,
        };

        Board {
            adc2,
            pressure_pin,
            modem,
            button_state,
            button,
            pump,
            boiller,
            boiller_state,
            three_way_valve,
            temperature_sensor_one,
            temperature_sensor_two,
            temperature_sensor_three,
        }
    }
    pub fn get_button_state(&self) -> bool {
        return self.button.is_high();
    }
    pub fn set_boiler(&mut self, on_off: BoillerState) -> Result<(), EspError> {
        match on_off {
            BoillerState::On => {
                let set_high = self.boiller.set_high();
                self.boiller_state = BoillerState::On;
                return set_high;
            }
            BoillerState::Off => {
                self.boiller.is_set_low();
                self.boiller_state = BoillerState::Off;
                return Ok(());
            }
        }
    }
}

impl<'a> fmt::Debug for Board<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Board")
            .field("modem", &"Modem") // Assuming Modem doesn't implement Debug
            .field("adc2", &"ADC2") // Assuming ADC2 doesn't implement Debug
            .field("gpio2", &"self.gpio2")
            .field("button_state", &self.button_state)
            .finish()
    }
}
