use core::convert::TryInto;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use log::{error, info, warn};

const SSID: &str = env!("WIFI_SSID");
const PASSWORD: &str = env!("WIFI_PASS");
const MAX_RETRIES: u8 = 10;

pub async fn connect_wifi(
    peripherals: Peripherals,
    sys_loop: EspSystemEventLoop,
    timer_service: EspTaskTimerService,
    _nvs: EspDefaultNvsPartition,
) -> anyhow::Result<AsyncWifi<EspWifi<'static>>> {
    // peripherals.modem abstrahiert die Hardware-Unterschiede weg
    let esp_wifi = EspWifi::new(peripherals.modem, sys_loop.clone(), None)?;
    let mut wifi = AsyncWifi::wrap(esp_wifi, sys_loop, timer_service)?;

    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
                                                                  password: PASSWORD.try_into().unwrap(),
                                                                  auth_method: AuthMethod::WPA2Personal,
                                                                  ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    info!("Starting WiFi...");
    wifi.start().await?;

    let mut attempts = 0;
    loop {
        attempts += 1;
        info!("Connecting to SSID: {} (Attempt {}/{})...", SSID, attempts, MAX_RETRIES);

        match wifi.connect().await {
            Ok(_) => {
                info!("Waiting for network interface...");
                match wifi.wait_netif_up().await {
                    Ok(_) => {
                        info!("WiFi connected successfully!");
                        return Ok(wifi);
                    }
                    Err(e) => warn!("Netif up failed: {:?}", e),
                }
            }
            Err(e) => warn!("Connection attempt failed: {:?}", e),
        }

        if attempts >= MAX_RETRIES {
            error!("FATAL: Failed to connect after {} attempts.", MAX_RETRIES);
            info!("Triggering hardware reset in 3 seconds...");
            FreeRtos::delay_ms(3000);
            esp_idf_svc::hal::reset::restart();
        }

        // Wait before next attempt
        FreeRtos::delay_ms(5000);
    }
}
