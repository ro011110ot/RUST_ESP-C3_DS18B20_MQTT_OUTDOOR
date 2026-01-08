mod ds18b20;
mod mqtt;
mod wifi;

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::{AnyIOPin, PinDriver};
// Added AnyIOPin here
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::reset::WakeupReason;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::mqtt::client::QoS;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, sntp::EspSntp};
use log::info;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    info!("Booting... Reason: {:?}", WakeupReason::get());

    // 1. WiFi & NTP Sync (Essential for time calculation)
    info!("Initializing WiFi...");
    let _wifi = block_on(wifi::connect_wifi(
        peripherals,
        sys_loop,
        timer_service,
        nvs,
    ))?;

    info!("Syncing time via NTP...");
    let _sntp = EspSntp::new_default()?;

    // Wait for sync to complete
    FreeRtos::delay_ms(5000);

    // 2. Calculate next sync point
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    // We want to measure at 0, 15, 30, 45 minutes
    let interval = 15 * 60; // 900 seconds
    let seconds_past_last_interval = now % interval;

    // IMPORTANT: If we are very close to an interval (e.g., within 30s),
    // we measure now. Otherwise, we sleep until the next one.
    if seconds_past_last_interval > 30 && WakeupReason::get() == WakeupReason::Unknown {
        let sleep_until_next = interval - seconds_past_last_interval;
        info!(
            "Not at interval. Syncing sleep: {}s remaining.",
            sleep_until_next
        );
        deep_sleep(sleep_until_next);
        return Ok(());
    }

    // 3. Sensor Measurement
    let ds_pin = PinDriver::input_output(unsafe { AnyIOPin::new(4) })?;
    let mut sensor = ds18b20::Ds18b20::new(ds_pin);
    FreeRtos::delay_ms(1000);

    if let Some(temp) = sensor.read_temp() {
        info!("Measurement successful: {:.2}°C", temp);

        // 4. MQTT Transmission
        let mut mqtt_client = mqtt::create_mqtt_client()?;
        FreeRtos::delay_ms(2000);

        let topic = format!("{}/DS18B20", env!("MQTT_TOPIC"));
        let payload = format!(r#"{{"id": "DS18B20_Outdoor", "Temp": {:.2}}}"#, temp);
        let _ = mqtt_client.publish(&topic, QoS::AtMostOnce, false, payload.as_bytes());

        FreeRtos::delay_ms(2000);
        info!("Data sent.");
    }

    // 5. Calculate sleep time until the NEXT exact 15m slot
    let now_after_work = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let sleep_secs = interval - (now_after_work % interval);

    info!("Work done. Sleeping {}s until next interval.", sleep_secs);
    deep_sleep(sleep_secs);

    Ok(())
}

fn deep_sleep(secs: u64) {
    unsafe {
        esp_idf_svc::sys::esp_sleep_enable_timer_wakeup(secs * 1_000_000);
        esp_idf_svc::sys::esp_deep_sleep_start();
    }
}
