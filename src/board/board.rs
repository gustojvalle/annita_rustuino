use core::fmt;
use esp_idf_hal::adc::ADC2;
use esp_idf_hal::gpio::{Gpio2, Gpio9, Input, PinDriver, Pull};
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::prelude::Peripherals;

pub struct Board<'a> {
    pub modem: Modem,
    pub adc2: ADC2,
    pub gpio2: Gpio2,
    pub button_state: bool,
    button: PinDriver<'a, Gpio9, Input>,
}
impl<'a> Board<'a> {
    pub fn init(p: Peripherals) -> Board<'a> {
        let adc2 = p.adc2;
        let modem = p.modem;
        let gpio2: Gpio2 = p.pins.gpio2;
        let button_pin = p.pins.gpio9;
        let mut button = PinDriver::input(button_pin).unwrap();
        button.set_pull(Pull::Down).unwrap();
        let button_state = button.is_high();
        Board {
            adc2,
            gpio2,
            modem,
            button_state,
            button,
        }
    }
    pub fn get_button_state(&self) -> bool {
        return self.button.is_high();
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

