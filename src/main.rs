mod ds18b20;
mod mqtt;
mod wifi;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{AnyIOPin, PinDriver};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::mqtt::client::QoS;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, sntp::EspSntp};
use log::{error, info, warn};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    info!("Booting ESP32-C3 Outdoor Node (Continuous Mode)...");

    // 1. Initial WiFi & NTP Sync
    let mut wifi = block_on(wifi::connect_wifi(
        peripherals,
        sys_loop.clone(),
        timer_service.clone(),
        nvs,
    ))?;

    info!("Syncing time via NTP...");
    let _sntp = EspSntp::new_default()?;

    // Wait for valid NTP time (Check against Jan 2026 threshold)
    while SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() < 1736340000 {
        info!("Waiting for NTP sync...");
        FreeRtos::delay_ms(2000);
    }
    info!("Time synchronized.");

    // Initialize DS18B20 Sensor on GPIO 4
    let ds_pin = PinDriver::input_output(unsafe { AnyIOPin::new(4) })?;
    let mut sensor = ds18b20::Ds18b20::new(ds_pin);

    // Initialize MQTT Client (Keep connection alive)
    let mut mqtt_client = mqtt::create_mqtt_client()?;

    let interval = 900; // 15 minutes
    let mut last_processed_slot = 0;

    info!("Entering main loop...");

    loop {
        // --- WIFI RECONNECT CHECK ---
        match wifi.is_up() {
            Ok(connected) => {
                if !connected {
                    warn!("WiFi link down! Attempting to reconnect...");
                    let _ = wifi.connect();
                    FreeRtos::delay_ms(5000);
                    continue; // Skip interval check until WiFi is back
                }
            }
            Err(e) => error!("WiFi health check error: {:?}", e),
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let current_slot = now - (now % interval);
        let seconds_past_slot = now % interval;

        // Trigger measurement in the 1-minute window of the 15m interval
        if current_slot > last_processed_slot && seconds_past_slot < 60 {
            info!("Interval reached (Sync Window)! Measuring DS18B20...");

            if let Some(temp) = sensor.read_temp() {
                info!("Temperature: {:.2}C", temp);

                let topic = format!("{}/DS18B20", env!("MQTT_TOPIC"));
                let payload = format!(r#"{{"id": "DS18B20_Outdoor", "Temp": {:.2}}}"#, temp);

                info!("Publishing with QoS 1 to {}...", topic);
                match mqtt_client.publish(&topic, QoS::AtLeastOnce, false, payload.as_bytes()) {
                    Ok(_) => {
                        info!("Publish successful. Waiting for network finalization...");
                        FreeRtos::delay_ms(5000); // Ensure network stack processes ACKs
                    }
                    Err(e) => error!("MQTT publish failed: {:?}", e),
                }
            } else {
                warn!("Sensor reading failed.");
            }

            last_processed_slot = current_slot;
        }

        // Prevent CPU starvation
        FreeRtos::delay_ms(1000);
    }
}
