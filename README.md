Hier ist die aktualisierte README.md. Ich habe alle Referenzen zum Deep-Sleep entfernt, die kontinuierliche
Synchronisation hinzugefügt und sichergestellt, dass alles gemäß deinen Vorgaben auf Englisch verfasst ist.

🌡️ ESP32-C3 Outdoor Temperature Node (Rust)
An asynchronous, high-precision IoT application built with Rust and ESP-IDF for the ESP32-C3 (RISC-V). This node
performs continuous, synchronized temperature measurements using a DS18B20 1-Wire sensor and publishes data via MQTT.

🚀 Features
Continuous Operation: Optimized for mains-powered deployment with a persistent execution loop (Deep-Sleep disabled).

Synchronized Measurement: Triggers data transmission exactly at XX:00, XX:15, XX:30, and XX:45 using NTP-synchronized
time.

Persistent MQTT Connection: Maintains a stable connection to the broker, reducing overhead and improving reliability
compared to wake-cycle logic.

QoS Level 1 Messaging: Ensures guaranteed delivery of sensor data through broker acknowledgment (AtLeastOnce).

Flash-Efficient Networking: WiFi NVS (Non-Volatile Storage) caching is disabled to prevent unnecessary flash wear.

DS18B20 1-Wire Implementation: Manual timing-accurate implementation for DS18B20 on GPIO 4.

JSON Payloads: Structured for seamless integration with dashboards or databases:

{"id": "DS18B20_Outdoor", "Temp": 22.50}

Environment Driven: Secure handling of credentials via .env and build.rs at compile time.

🛠 Hardware Setup
Microcontroller: ESP32-C3 (e.g., SuperMini or DevKit-C).

Sensor: DS18B20 (Waterproof probe recommended).

Wiring:

VCC: 3.3V

GND: Ground

Data: GPIO 4 (Requires a 4.7kΩ pull-up resistor between VCC and Data).

📋 Prerequisites & Toolchain
Rust & ESP-IDF:

Bash

espup install

# Ensure the riscv32imc-esp-espidf target is installed

Linker: Requires ldproxy for proper ESP-IDF integration.

Editor: Developed using micro on Manjaro/Debian systems.

⚙️ Configuration
Create a .env file in the project root to manage your credentials:

Code-Snippet

WIFI_SSID="Your_SSID"
WIFI_PASS="Your_Password"
MQTT_BROKER="mqtt://your.broker.ip:1883"
MQTT_USER="your_user"
MQTT_PASS="your_password"
MQTT_TOPIC="Sensors/Outdoor"
🔨 Build and Flash
The project is configured to use /dev/ttyACM0 (internal JTAG/Serial).

Bash

# Clean build is recommended when changing environment variables

cargo clean
cargo run
🔄 Execution Logic
The device follows a continuous, synchronized execution flow:

Boot: Initializes peripherals and sets up the English-language logging system.

Network: Connects to WiFi and synchronizes internal RTC via NTP.

MQTT Init: Establishes a persistent connection to the specified broker.

Sync Loop:

Monitors the system clock every second.

If a 15-minute interval (00, 15, 30, 45) is reached, it enters the 1-minute measurement window.

Measure & Publish: Performs thermal conversion and sends a JSON payload with QoS 1.

Wait: Reverts to idle state until the next interval while keeping the connection alive.

🔍 Implementation Notes
Language Standard: All code comments, log outputs (info!, error!), and documentation are strictly in English.

Synchronization: Uses now % 900 logic to ensure this node aligns perfectly with other sensors (e.g., DHT11 nodes) on the
dashboard.

NVS Protection: EspWifi is initialized without NVS to protect the flash memory from frequent write operations.

📄 License
Distributed under the MIT License.