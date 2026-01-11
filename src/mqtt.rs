use esp_idf_svc::mqtt::client::*;
use log::{error, info};
use std::time::Duration;

/// Creates and configures the MQTT client with environment variables.
pub fn create_mqtt_client() -> anyhow::Result<EspMqttClient<'static>> {
    let url = env!("MQTT_BROKER");
    let user = env!("MQTT_USER");
    let pass = env!("MQTT_PASS");
    let client_id = env!("MQTT_CLIENT_ID");

    info!("Connecting to MQTT broker: {}", url);

    let mqtt_config = MqttClientConfiguration {
        client_id: Some(client_id),
        username: Some(user),
        password: Some(pass),
        // Robust keep-alive for outdoor stability
        keep_alive_interval: Some(Duration::from_secs(120)),
        network_timeout: Duration::from_secs(15),
        reconnect_timeout: Some(Duration::from_secs(5)),
        ..Default::default()
    };

    let client = EspMqttClient::new_cb(url, &mqtt_config, move |event| match event.payload() {
        EventPayload::Connected(_) => info!("MQTT Status: Connected"),
        EventPayload::Disconnected => info!("MQTT Status: Disconnected"),
        EventPayload::Published(id) => info!("MQTT Message published (ID: {})", id),
        EventPayload::Error(e) => error!("MQTT Error encountered: {:?}", e),
        _ => {}
    })?;

    Ok(client)
}
