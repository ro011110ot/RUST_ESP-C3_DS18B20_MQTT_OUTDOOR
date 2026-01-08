use esp_idf_svc::mqtt::client::*;
use log::{error, info};
use std::time::Duration;

pub fn create_mqtt_client() -> anyhow::Result<EspMqttClient<'static>> {
    // Load environment variables (ensure MQTT_BROKER starts with mqtt://)
    let url = env!("MQTT_BROKER");
    let user = env!("MQTT_USER");
    let pass = env!("MQTT_PASS");
    let client_id = env!("MQTT_CLIENT_ID");
    info!("Connecting to MQTT broker (Unencrypted): {}", url);

    let mqtt_config = MqttClientConfiguration {
        client_id: Some(client_id),
        username: Some(user),
        password: Some(pass),
        keep_alive_interval: Some(Duration::from_secs(60)),
        // FIX: network_timeout expects Duration, not Some(Duration)
        network_timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let client = EspMqttClient::new_cb(url, &mqtt_config, move |event| match event.payload() {
        EventPayload::Connected(_) => info!("MQTT Status: Connected"),
        EventPayload::Disconnected => info!("MQTT Status: Disconnected"),
        EventPayload::Subscribed(id) => info!("MQTT Subscription confirmed: {}", id),
        EventPayload::Published(id) => info!("MQTT Message published (ID: {})", id),
        EventPayload::Error(e) => error!("MQTT Error encountered: {:?}", e),
        _ => {}
    })?;

    Ok(client)
}
