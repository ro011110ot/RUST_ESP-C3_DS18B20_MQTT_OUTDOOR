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

    info!("Booting Node in Continuous Mode (Deep Sleep disabled)...");

    // 1. WiFi & NTP Sync
    info!("Initializing WiFi...");
    let _wifi = block_on(wifi::connect_wifi(
        peripherals,
        sys_loop,
        timer_service,
        nvs,
    ))?;

    info!("Syncing time via NTP...");
    let _sntp = EspSntp::new_default()?;

    // Wait for valid NTP time
    while SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() < 1736340000 {
        info!("Waiting for NTP sync...");
        FreeRtos::delay_ms(2000);
    }
    info!("Time synchronized.");

    // Initialize Sensor Pin
    let ds_pin = PinDriver::input_output(unsafe { AnyIOPin::new(4) })?;
    let mut sensor = ds18b20::Ds18b20::new(ds_pin);

    // Initialize MQTT Client once (keep connection alive)
    let mut mqtt_client = mqtt::create_mqtt_client()?;

    let interval = 900; // 15 minutes
    let mut last_processed_slot = 0;

    info!("Entering main loop...");

    loop {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let current_slot = now - (now % interval);
        let seconds_past_slot = now % interval;

        // Trigger measurement if in the first 60s of a new 15m slot
        if current_slot > last_processed_slot && seconds_past_slot < 60 {
            info!("Interval reached! Starting synchronized measurement...");

            if let Some(temp) = sensor.read_temp() {
                info!("Temperature: {:.2}C", temp);

                let topic = format!("{}/DS18B20", env!("MQTT_TOPIC"));
                let payload = format!(r#"{{"id": "DS18B20_Outdoor", "Temp": {:.2}}}"#, temp);

                info!("Publishing to {}...", topic);
                if let Err(e) =
                    mqtt_client.publish(&topic, QoS::AtLeastOnce, false, payload.as_bytes())
                {
                    error!("MQTT publish failed: {:?}", e);
                } else {
                    info!("Publish successful.");
                }
            } else {
                warn!("Sensor read failed.");
            }

            last_processed_slot = current_slot;
        }

        // Small delay to prevent CPU hogging
        FreeRtos::delay_ms(1000);
    }
}
