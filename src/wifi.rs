use core::convert::TryInto;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use log::info;

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");

pub async fn connect_wifi(
    peripherals: Peripherals,
    sys_loop: EspSystemEventLoop,
    timer_service: EspTaskTimerService,
    _nvs: EspDefaultNvsPartition,
) -> anyhow::Result<AsyncWifi<EspWifi<'static>>> {
    // WiFi initialized without NVS storage for faster connection
    let esp_wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), None)?;
    let mut wifi = AsyncWifi::wrap(esp_wifi, sys_loop, timer_service)?;

    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    info!("Starting WiFi (NVS storage disabled)...");
    wifi.start().await?;

    info!("Connecting to SSID: {}...", SSID);
    wifi.connect().await?;

    info!("Waiting for network interface (IP address)...");
    wifi.wait_netif_up().await?;

    Ok(wifi)
}
