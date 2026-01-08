# 🌡️ ESP32-C3 Outdoor Temperature Node (Rust)

An asynchronous, ultra-low-power IoT application built with **Rust** and **ESP-IDF** for the **ESP32-C3** (RISC-V). This
node measures outdoor temperature using a **DS18B20** 1-Wire sensor and publishes data via **MQTT** before entering a
deep sleep cycle.

---

## 🚀 Features

- **Deep Sleep Architecture**: Maximizes battery life by using a 15-minute measurement cycle with only ~5µA consumption
  during sleep.
- **Flash-Efficient Networking**: WiFi NVS (Non-Volatile Storage) caching is disabled (`Storage::Ram`) to prevent flash
  wear during frequent wake cycles.
- **Async WiFi & MQTT**: Non-blocking network stack using `esp-idf-svc` and `embedded-svc`.
- **DS18B20 Support**: Precise temperature readings via a manual 1-Wire implementation on **GPIO 4**.
- **JSON Payloads**: Optimized for easy ingestion by Home Assistant or Debian-based loggers:  
  `{"id": "DS18B20_Outdoor", "Temp": 22.50}`
- **Environment Driven**: Secure credential handling via `.env` and `build.rs` at compile time.

---

## 🛠 Hardware Setup

- **Microcontroller**: ESP32-C3 (e.g., SuperMini or DevKit-C).
- **Sensor**: DS18B20 (Waterproof version recommended).
- **Wiring**:
    - **VCC**: 3.3V
    - **GND**: Ground
    - **Data**: **GPIO 4** (Requires a **4.7kΩ pull-up resistor** between VCC and Data).

---

## 📋 Prerequisites & Toolchain

1. **Rust & ESP-IDF**:
   ```bash
   espup install
   # Ensure the riscv32imc-esp-espidf target is installed

Linker: Requires ldproxy for proper ESP-IDF integration.

Editor: Developed using micro on Manjaro/Debian systems.

⚙️ Configuration
Create a .env file in the project root to manage your credentials securely:

Code-Snippet

```
WIFI_SSID="Your_SSID"
WIFI_PASS="Your_Password"
MQTT_HOST="your.broker.ip"
MQTT_USER="your_user"
MQTT_PASS="your_password"
MQTT_TOPIC="Sensors/Outdoor"
```

# 🔨 Build and Flash

The project is pre-configured in .cargo/config.toml to use /dev/ttyACM0.

Bash

# Clean build is recommended when changing environment variables

```
cargo clean
cargo run
```

# 🔋 Power Management Cycle

The device follows a highly efficient execution flow:

Wake: RTC Timer triggers a full system boot.

Initialize: Sets up peripherals and logging.

Network: Connects to WiFi (RAM storage only, no flash wear).

Measure: Performs DS18B20 thermal conversion and data read.

Publish: Establishes MQTT connection and sends JSON data.

Sleep: Enters Deep Sleep for 900 seconds (15 mins). The CPU is powered down.

# 🔍 Implementation Notes

__pender Fix: Included a manual no_mangle stub in main.rs to resolve linking conflicts between esp-idf-svc and
embassy-executor.

NVS Protection: By initializing EspWifi with None for the NVS partition, we ensure that the WiFi stack does not perform
thousands of write cycles to the flash memory over the year.

# 📄 License

Distributed under the MIT