// SPDX-License-Identifier: MIT OR Apache-2.0

/// Simple example of a Thing mapping a sensor using
/// the [esp-rust-board](https://github.com/esp-rs/esp-rust-board).
/// The code is based on the http-server example from the
/// [std-training](https://esp-rs.github.io/std-training).
use anyhow::Result;
use core::str;
use embedded_svc::{http::Method, io::Write};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        i2c::{I2cConfig, I2cDriver},
        prelude::*,
    },
    http::server::{Configuration, EspHttpServer},
};
use shtcx::{self, shtc3, PowerMode};
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};
use wot_esp_demo::wifi;
use wot_td::{builder::*, Thing};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}
fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    // The constant `CONFIG` is auto-generated by `toml_config`.
    let app_config = CONFIG;
    // Connect to the Wi-Fi network
    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    )?;

    // Initialize temperature sensor
    let sda = peripherals.pins.gpio10;
    let scl = peripherals.pins.gpio8;
    let i2c = peripherals.i2c0;
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;
    let mut sht = shtc3(i2c);
    let device_id = sht.device_identifier().unwrap();
    let sensor_main = Arc::new(Mutex::new(sht));
    let sensor = sensor_main.clone();
    sensor
        .lock()
        .unwrap()
        .start_measurement(PowerMode::NormalMode)
        .unwrap();

    println!("Device ID SHTC3: {:#02x}", device_id);

    let base_ip_address = _wifi.sta_netif().get_ip_info()?.ip;
    let base_uri = format!("http://{}", base_ip_address);

    // Build the thing description
    let td = Thing::builder("shtc3")
        .finish_extend()
        .id(format!("urn:shtc3/{device_id:#02x}"))
        .base(base_uri)
        .description("Example Thing exposing a shtc3 sensor")
        .security(|builder| builder.no_sec().required().with_key("nosec_sc"))
        .property("temperature", |p| {
            p.finish_extend_data_schema()
                .attype("TemperatureProperty")
                .title("Temperature")
                .description("Current temperature")
                .form(|f| {
                    f.href("/properties/temperature")
                        .op(wot_td::thing::FormOperation::ReadProperty)
                })
                .number()
                .read_only()
        })
        .property("humidity", |p| {
            p.finish_extend_data_schema()
                .attype("HumidityProperty")
                .title("Humidity")
                .description("Current humidity")
                .form(|f| {
                    f.href("/properties/humidity")
                        .op(wot_td::thing::FormOperation::ReadProperty)
                })
                .number()
                .read_only()
        })
        .build()?;

    // Set the HTTP server
    let mut server = EspHttpServer::new(&Configuration::default())?;

    // Serve the TD from the root
    server.fn_handler("/", Method::Get, |request| -> Result<()> {
        let json = serde_json::to_vec(&td)?;
        let mut response = request.into_ok_response()?;
        response.write_all(&json)?;
        Ok(())
    })?;

    // Redirect the Well-Known URI to / so we can leave Thing::base empty.
    server.fn_handler("/.well-known/wot", Method::Get, |request| -> Result<()> {
        let mut response = request.into_response(302, None, &[("Location", "/")])?;
        response.flush()?;
        Ok(())
    })?;

    // TemperatureProperty
    let temp_sensor = sensor.clone();
    server.fn_handler(
        "/properties/temperature",
        Method::Get,
        move |request| -> Result<()> {
            let temp_val = temp_sensor
                .lock()
                .unwrap()
                .get_measurement_result()
                .unwrap()
                .temperature
                .as_degrees_celsius();
            let json = format!("{temp_val:.2}");
            let mut response = request.into_ok_response()?;
            response.write_all(json.as_bytes())?;
            Ok(())
        },
    )?;

    // HumidityProperty
    server.fn_handler(
        "/properties/humidity",
        Method::Get,
        move |request| -> Result<()> {
            let humi_val = sensor
                .lock()
                .unwrap()
                .get_measurement_result()
                .unwrap()
                .humidity
                .as_percent();

            let json = format!("{humi_val:.1}");
            let mut response = request.into_ok_response()?;
            response.write_all(json.as_bytes())?;
            Ok(())
        },
    )?;

    println!("Server awaiting connection");

    // Prevent program from exiting
    loop {
        sensor_main
            .lock()
            .unwrap()
            .start_measurement(PowerMode::NormalMode)
            .unwrap();
        sleep(Duration::from_millis(1000));
    }
}
