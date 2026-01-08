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

    info!("Booting ESP32-C3 Outdoor Node (Enhanced Recovery)...");

    // 1. Initial WiFi Setup
    let mut wifi = block_on(wifi::connect_wifi(
        peripherals,
        sys_loop.clone(),
        timer_service.clone(),
        nvs,
    ))?;

    // 2. NTP Sync
    let _sntp = EspSntp::new_default()?;
    while SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() < 1736340000 {
        info!("Waiting for NTP sync...");
        FreeRtos::delay_ms(2000);
    }
    info!("Time synchronized.");

    // 3. Sensor & MQTT Initialization
    // GPIO 4 for DS18B20
    let ds_pin = PinDriver::input_output(unsafe { AnyIOPin::new(4) })?;
    let mut sensor = ds18b20::Ds18b20::new(ds_pin);
    let mut mqtt_client = mqtt::create_mqtt_client()?;

    let interval = 900; // 15 minutes
    let mut last_processed_slot = 0;
    let mut reconnect_counter = 0;

    info!("Entering main loop...");

    loop {
        // --- WIFI RECOVERY LOGIC ---
        match wifi.is_up() {
            Ok(connected) => {
                if !connected {
                    reconnect_counter += 1;
                    warn!(
                        "WiFi link lost (Attempt {}). Reconnecting...",
                        reconnect_counter
                    );

                    if reconnect_counter > 20 {
                        error!("WiFi recovery failed. Restarting chip...");
                        esp_idf_svc::hal::reset::restart();
                    } else if reconnect_counter % 5 == 0 {
                        info!("Cycling WiFi stack to clear driver state...");
                        let _ = wifi.stop();
                        FreeRtos::delay_ms(1000);
                        let _ = wifi.start();
                    }

                    let _ = wifi.connect();
                    FreeRtos::delay_ms(5000);
                    continue;
                } else if reconnect_counter > 0 {
                    info!("WiFi connection re-established!");
                    reconnect_counter = 0;
                }
            }
            Err(e) => error!("WiFi health check error: {:?}", e),
        }

        // --- MEASUREMENT LOGIC ---
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let current_slot = now - (now % interval);
        let seconds_past_slot = now % interval;

        // Check for 15-minute interval
        if current_slot > last_processed_slot && seconds_past_slot < 60 {
            info!("Interval reached! Starting DS18B20 measurement...");

            if let Some(temp) = sensor.read_temp() {
                info!("Temperature: {:.2}C", temp);

                let topic = format!("{}/DS18B20", env!("MQTT_TOPIC"));
                let payload = format!(r#"{{"id": "DS18B20_Outdoor", "Temp": {:.2}}}"#, temp);

                info!("Publishing to MQTT with QoS 1...");
                match mqtt_client.publish(&topic, QoS::AtLeastOnce, false, payload.as_bytes()) {
                    Ok(_) => {
                        info!("Transmission successful.");
                        // Brief delay for network stability
                        FreeRtos::delay_ms(5000);
                    }
                    Err(e) => error!("MQTT publish failed: {:?}", e),
                }
            } else {
                warn!("Sensor read failed - check wiring and 4.7k resistor.");
            }

            last_processed_slot = current_slot;
        }

        // Prevent CPU starvation
        FreeRtos::delay_ms(1000);
    }
}
