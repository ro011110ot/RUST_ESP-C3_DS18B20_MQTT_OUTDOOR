// src/ds18b20.rs
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::{IOPin, PinDriver};
use log::error;

pub struct Ds18b20<'a, T: IOPin> {
    pin: PinDriver<'a, T, esp_idf_svc::hal::gpio::InputOutput>,
}

impl<'a, T: IOPin> Ds18b20<'a, T> {
    pub fn new(pin: PinDriver<'a, T, esp_idf_svc::hal::gpio::InputOutput>) -> Self {
        Self { pin }
    }

    fn reset(&mut self) -> bool {
        self.pin.set_low().unwrap();
        Ets::delay_us(480);
        self.pin.set_high().unwrap();
        Ets::delay_us(70);
        let present = self.pin.is_low();
        Ets::delay_us(410);
        present
    }

    fn write_bit(&mut self, bit: bool) {
        self.pin.set_low().unwrap();
        if bit {
            Ets::delay_us(10);
            self.pin.set_high().unwrap();
            Ets::delay_us(55);
        } else {
            Ets::delay_us(65);
            self.pin.set_high().unwrap();
            Ets::delay_us(5);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        for i in 0..8 {
            self.write_bit((byte >> i) & 1 == 1);
        }
    }

    fn read_bit(&mut self) -> bool {
        self.pin.set_low().unwrap();
        Ets::delay_us(3);
        self.pin.set_high().unwrap();
        Ets::delay_us(10);
        let bit = self.pin.is_low();
        Ets::delay_us(53);
        !bit // Invertiert, da Pull-up
    }

    fn read_byte(&mut self) -> u8 {
        let mut byte = 0u8;
        for i in 0..8 {
            if self.read_bit() {
                byte |= 1 << i;
            }
        }
        byte
    }

    pub fn read_temp(&mut self) -> Option<f32> {
        if !self.reset() {
            error!("DS18B20: Kein Sensor gefunden (Reset fehlgeschlagen)");
            return None;
        }

        self.write_byte(0xCC); // Skip ROM
        self.write_byte(0x44); // Start Conversion

        // Warte auf Ende der Konvertierung (parasitäre Speisung nicht unterstützt)
        Ets::delay_us(750_000);

        if !self.reset() { return None; }

        self.write_byte(0xCC); // Skip ROM
        self.write_byte(0xBE); // Read Scratchpad

        let temp_lsb = self.read_byte();
        let temp_msb = self.read_byte();

        let temp_raw = ((temp_msb as i16) << 8) | (temp_lsb as i16);
        Some(temp_raw as f32 / 16.0)
    }
}