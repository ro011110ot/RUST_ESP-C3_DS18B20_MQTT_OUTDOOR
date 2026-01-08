use esp_idf_svc::mqtt::client::*;
use log::{error, info};
use std::time::Duration;

pub fn create_mqtt_client() -> anyhow::Result<EspMqttClient<'static>> {
    let url = env!("MQTT_BROKER");
    let user = env!("MQTT_USER");
    let pass = env!("MQTT_PASS");

    info!("Connecting to MQTT broker: {}", url);

    let mqtt_config = MqttClientConfiguration {
        client_id: Some("esp32_c3_outdoor"),
        username: Some(user),
        password: Some(pass),
        keep_alive_interval: Some(Duration::from_secs(60)),
        network_timeout: Duration::from_secs(10),
        ..Default::default()
    };

    let client = EspMqttClient::new_cb(url, &mqtt_config, move |event| match event.payload() {
        EventPayload::Connected(_) => info!("MQTT Status: Connected"),
        EventPayload::Disconnected => info!("MQTT Status: Disconnected"),
        EventPayload::Published(id) => info!("MQTT Message published (ID: {})", id),
        EventPayload::Error(e) => error!("MQTT Error: {:?}", e),
        _ => {}
    })?;

    Ok(client)
}
