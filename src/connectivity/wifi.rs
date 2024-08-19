use anyhow::{bail, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

use crate::board::board::Board;

pub fn wifi<'a>(
    ssid: &str,
    pass: &str,
    board: &'a mut Board,
    sysloop: EspSystemEventLoop,
) -> Result<EspWifi<'a>> {
    let mut auth_method = AuthMethod::WPA2Personal;
    if ssid.is_empty() {
        bail!("Missing WiFi name")
    }
    if pass.is_empty() {
        auth_method = AuthMethod::None;
        print!("Wifi password is empty");
    }

    let mut esp_wifi = EspWifi::new(&mut board.modem, sysloop.clone(), None)?;

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    print!("Starting wifi...");

    wifi.start()?;

    print!("Scanning...");

    let ap_infos = wifi.scan()?;

    print!(
        "Scanned results {:?}",
        ap_infos.clone().into_iter().map(|network| network.ssid)
    );

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        print!(
            "Found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        print!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid
            .try_into()
            .expect("Could not parse the given SSID into WiFi config"),
        password: pass
            .try_into()
            .expect("Could not parse the given password into WiFi config"),
        channel,
        auth_method,
        ..Default::default()
    }))?;

    print!("Connecting wifi...");

    wifi.connect()?;

    print!("Waiting for DHCP lease...");

    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    print!("Wifi DHCP info: {:?}", ip_info);

    Ok(esp_wifi)
}

fn bluetooth() {}
