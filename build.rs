fn main() {
    // Sorgt für die korrekten ESP-IDF Pfade
    embuild::espidf::sysenv::output();

    // .env Datei laden
    dotenvy::dotenv().ok();

    // Liste der Variablen, die an den Compiler (env!) weitergegeben werden sollen
    let env_vars = [
        "WIFI_SSID",
        "WIFI_PASS",
        "MQTT_BROKER",
        "MQTT_USER",
        "MQTT_PASS",
        "MQTT_TOPIC",
        "MQTT_CLIENT_ID",
    ];

    for var in env_vars {
        if let Ok(value) = std::env::var(var) {
            println!("cargo:rustc-env={}={}", var, value);
        }
    }

    // Build-Script neu ausführen, wenn sich die .env ändert
    println!("cargo:rerun-if-changed=.env");
}
