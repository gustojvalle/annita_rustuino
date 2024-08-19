use anyhow::Error;
use core::fmt;
use esp32_nimble::utilities::BleUuid;
use esp32_nimble::BLEService;
use esp32_nimble::{
    utilities::mutex, uuid128, BLEAdvertisementData, BLECharacteristic, BLEDevice, BLEServer,
    NimbleProperties,
};
use esp_idf_hal::adc::ADC2;
use esp_idf_hal::gpio::{
    AnyInputPin, AnyOutputPin, Gpio12, Gpio17, Gpio18, Gpio19, Gpio2, Gpio20, Gpio21, Gpio22, Gpio25, Input, Output, Pin, PinDriver, Pull
};
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_sys::{ble_uuid128_t, ble_uuid_any_t};
use log::info;
use rbd_dimmer::{DevicesDimmerManager, DevicesDimmerManagerConfig, DimmerDevice};
use serde_json::Map;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::actuators::pump::PumpConfig;
use crate::connectivity::bt::ble_server;

#[derive(Debug, PartialEq)]
pub enum BoillerState {
    On,
    Off,
}

type SharedLed<'a> = Arc<Mutex<PinDriver<'a, Gpio2, esp_idf_hal::gpio::Output>>>;

// Initialize the shared LED state. You should initialize this at a place where you can ensure that the board is not borrowed mutably.

pub struct Board<'a> {
    pub modem: Modem,
    pub adc2: ADC2,
    pub pressure_pin: Gpio12,
    pub button_state: bool,
    pub pump: Gpio17,
    pub boiller: PinDriver<'a, Gpio18, Output>,
    pub three_way_valve: Gpio19,
    pub temperature_sensor_one: Gpio20,
    pub temperature_sensor_two: Gpio21,
    pub temperature_sensor_three: Gpio22,
    pub boiller_state: BoillerState,
    button: PinDriver<'a, Gpio25, Input>,
    pub pump_config: PumpConfig,
    pub ble_device: &'a mut BLEDevice, // pub bluetooth: BLEClient,
    pub ble_services: HashMap<String, Arc<mutex::Mutex<BLEService>>>,
    pub ble_characteristics: HashMap<String, Arc<mutex::Mutex<BLECharacteristic>>>,
    pub onboard_led:Arc<Mutex<PinDriver<'a, Gpio2, esp_idf_hal::gpio::Output>>> 
}
impl<'a> Board<'a> {
    pub fn init() -> Result<Board<'a>, Error> {
        let mut ble_services = HashMap::new();
        let ble_characteristics = HashMap::new();
        let ble_device = setup_ble_server(&mut ble_services);
        thread::sleep(Duration::from_secs(5));
        let p = Peripherals::take().unwrap();
        let adc2 = p.adc2;
        let modem = p.modem;
        let pressure_pin = p.pins.gpio12;
        let button_pin = p.pins.gpio25;
        let pump = p.pins.gpio17;
        let boiller_pin = p.pins.gpio18;
        let three_way_valve = p.pins.gpio19;
        let temperature_sensor_one = p.pins.gpio20;
        let temperature_sensor_two = p.pins.gpio21;
        let temperature_sensor_three = p.pins.gpio22;
        let onboard_led = Arc::new(Mutex::new(PinDriver::output(p.pins.gpio2)?));

        //
        let mut button = PinDriver::input(button_pin).unwrap();
        button.set_pull(Pull::Down).unwrap();
        let button_state = button.is_high();
        let mut boiller = PinDriver::output(boiller_pin).unwrap();
        let boiller_state = match boiller.set_high() {
            Ok(()) => BoillerState::On,
            Err(_) => BoillerState::Off,
        };
        let zc_pin = p.pins.gpio33;
        let d0_pin = p.pins.gpio23;
        let pump_config = PumpConfig::default();

        // Create the zero-crossing pin and control pin
        let zc = unsafe { AnyInputPin::new(zc_pin.pin()) };
        let d0 = unsafe { AnyOutputPin::new(d0_pin.pin()) };

        let zc_driver = PinDriver::input(zc);
        // Setup PSM with the zero-crossing pin and control pin
        setup_psm(zc_driver.unwrap(), PinDriver::output(d0).unwrap());

        Ok(Board {
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
            pump_config,
            ble_device,
            ble_services,
            ble_characteristics,
            onboard_led
        })
    }
    pub fn get_button_state(&self) -> bool {
        return self.button.is_high();
    }
    // TODO figure why the boiler pin is failign
    // pub fn set_boiler(&mut self, on_off: BoillerState) -> Result<(), EspError> {
    //     match on_off {
    //         BoillerState::On => {
    //             let set_high = self.boiller.set_high();
    //             self.boiller_state = BoillerState::On;
    //             return set_high;
    //         }
    //         BoillerState::Off => {
    //             self.boiller.is_set_low();
    //             self.boiller_state = BoillerState::Off;
    //             return Ok(());
    //         }
    //     }
    // }

    pub fn set_ble_service(
        &mut self,
        uuid_service: BleUuid,
        service_name: &str,
    ) -> Arc<mutex::Mutex<BLEService>> {
        let server = self.ble_device.get_server();
        let ble_advertiser = self.ble_device.get_advertising();

        let new_service = server.create_service(uuid_service);

        ble_advertiser
            .lock()
            .set_data(
                BLEAdvertisementData::new()
                    .name(service_name)
                    .add_service_uuid(uuid_service),
            )
            .unwrap();
        self.ble_services
            .insert(String::from(service_name), new_service.clone());
        new_service
    }
    pub fn set_ble_characteristic(
        &mut self,
        service: Arc<mutex::Mutex<BLEService>>,
        uuid_characteristic: BleUuid,
        characteristic_name: &str,
        properties: NimbleProperties,
        initial_value: &[u8],
    ) -> Arc<mutex::Mutex<BLECharacteristic>> {
        let ble_advertiser = self.ble_device.get_advertising();
        // Create a characteristic to associate with created service
        //
        let new_characteristic = service
            .lock()
            .create_characteristic(uuid_characteristic, properties);

        // Modify characteristic value
        new_characteristic.lock().set_value(initial_value).notify();

        self.ble_characteristics.insert(
            String::from(characteristic_name),
            new_characteristic.clone(),
        );
        // Configure Advertiser Data
        new_characteristic
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

fn setup_psm(
    zero_crossing_pin: PinDriver<'static, AnyInputPin, Input>,
    d0_pin: PinDriver<'static, AnyOutputPin, Output>,
) {
    let id = 0;
    let d = DimmerDevice::new(id, d0_pin);
    // Create Power management
    let _ddm = DevicesDimmerManager::init(DevicesDimmerManagerConfig::default_50_hz(
        zero_crossing_pin,
        vec![d],
    ))
    .unwrap();
}

fn setup_ble_server(
    ble_services: &mut HashMap<String, Arc<mutex::Mutex<BLEService>>>,
) -> &'static mut BLEDevice {
    // Take ownership of device
    let ble_device = BLEDevice::take();

    // Obtain handle for peripheral advertiser
    let ble_advertiser = ble_device.get_advertising();

    // Obtain handle for server
    let server = ble_device.get_server();

    // Define server connect behaviour
    server.on_connect(|server, clntdesc| {
        // Print connected client data
        println!("{:?}", clntdesc);
        // Update connection parameters
        server
            .update_conn_params(clntdesc.conn_handle(), 24, 48, 0, 60)
            .unwrap();
    });

    // Define server disconnect behaviour
    server.on_disconnect(|_desc, _reason| {
        println!("Disconnected, back to advertising");
    });

    let heartbeat_uuid = uuid128!("9b574847-f706-436c-bed7-fc01eb0965c1");
    let heartbeat_service = server.create_service(heartbeat_uuid);
    ble_services.insert(String::from("heartbeat"), heartbeat_service.clone());

    ble_advertiser
        .lock()
        .set_data(
            BLEAdvertisementData::new()
                .name("ESP32 Server heartbeat")
                .add_service_uuid(heartbeat_uuid),
        )
        .unwrap();


    ble_device
}
