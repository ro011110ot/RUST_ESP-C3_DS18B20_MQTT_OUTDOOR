use esp_idf_svc::mqtt::client::*;
use log::info;
use std::time::Duration;

pub fn create_mqtt_client() -> anyhow::Result<EspMqttClient<'static>> {
    // These are now successfully loaded via build.rs from your .env
    let url = env!("MQTT_BROKER");
    let user = env!("MQTT_USER");
    let pass = env!("MQTT_PASS");

    let mqtt_config = MqttClientConfiguration {
        client_id: Some("esp32_idf_rust"),
        username: Some(user),
        password: Some(pass),
        keep_alive_interval: Some(Duration::from_secs(30)),
        ..Default::default()
    };

    let client = EspMqttClient::new_cb(url, &mqtt_config, move |event| {
        match event.payload() {
            EventPayload::Connected(_) => info!("MQTT Connected"),
            EventPayload::Subscribed(id) => info!("MQTT Subscribed ID: {}", id),
            // Corrected for v0.51 struct variant
            EventPayload::Received { data, .. } => {
                info!("MQTT Message received: {} bytes", data.len());
            }
            _ => info!("MQTT Event: {:?}", event.payload()),
        }
    })?;

    Ok(client)
}
