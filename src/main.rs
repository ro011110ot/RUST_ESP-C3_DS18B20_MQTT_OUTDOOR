mod ds18b20; // Die neue Datei einbinden
mod mqtt;
mod wifi;

use esp_idf_svc as _;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::reset::WakeupReason;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::mqtt::client::QoS;
use esp_idf_svc::sys::esp_deep_sleep_start;
use esp_idf_svc::sys::esp_sleep_enable_timer_wakeup;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use log::{error, info};

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    const BASE_TOPIC: &str = env!("MQTT_TOPIC");

    info!("Wache auf... Grund: {:?}", WakeupReason::get());

    // 1. WiFi verbinden
    let _wifi = block_on(wifi::connect_wifi(
        peripherals,
        sys_loop,
        timer_service,
        nvs,
    ))?;

    // 2. Sensor initialisieren & Messen
    let ds_pin = PinDriver::input_output(unsafe { esp_idf_svc::hal::gpio::AnyIOPin::new(4) })?;
    let mut sensor = ds18b20::Ds18b20::new(ds_pin);

    // Kleiner Delay damit der Sensor stabil ist
    FreeRtos::delay_ms(1000);

    if let Some(temp) = sensor.read_temp() {
        // 3. MQTT Verbindung aufbauen
        let mut mqtt_client = mqtt::create_mqtt_client()?;

        // Kurz warten auf Connection (Event-basiert)
        FreeRtos::delay_ms(1000);

        let topic = format!("{}/DS18B20", BASE_TOPIC);
        let payload = format!(r#"{{"id": "DS18B20_Outdoor", "Temp": {:.2}}}"#, temp);

        info!("Publishing: {}", payload);
        mqtt_client.publish(&topic, QoS::AtMostOnce, false, payload.as_bytes())?;

        info!("Daten erfolgreich gesendet!");

        // 4. MQTT sauber trennen
        drop(mqtt_client);
    } else {
        error!("Sensor konnte nicht gelesen werden.");
    }

    // 5. Deep Sleep vorbereiten
    let sleep_time_secs = 15 * 60; // 15 Minuten
    info!("Gehe in Deep Sleep für {} Sekunden...", sleep_time_secs);

    unsafe {
        // Zeit in Mikrosekunden (µs)
        esp_sleep_enable_timer_wakeup(sleep_time_secs * 1_000_000);
        esp_deep_sleep_start();
    }

    // Dieser Teil wird nie erreicht
    #[allow(unreachable_code)]
    Ok(())
}
