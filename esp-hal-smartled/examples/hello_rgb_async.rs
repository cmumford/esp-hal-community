//! Asynchronous RGB LED Demo
//!
//! This example drives an SK68XX RGB LED, which is connected to a pin on the
//! official DevKits.
//!
//! The demo will leverage the [`smart_leds`](https://crates.io/crates/smart-leds)
//! crate functionality to circle through the HSV hue color space (with
//! saturation and value both at 255). Additionally, we apply a gamma correction
//! and limit the brightness to 10 (out of 255).
//!
//! The following wiring is assumed for ESP32:
//! - LED => GPIO33
//! The following wiring is assumed for ESP32C3:
//! - LED => GPIO8
//! The following wiring is assumed for ESP32C6, ESP32H2:
//! - LED => GPIO8
//! The following wiring is assumed for ESP32S2:
//! - LED => GPIO18
//! The following wiring is assumed for ESP32S3:
//! - LED => GPIO48

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{rmt::Rmt, time::Rate, timer::timg::TimerGroup, Config};
use esp_hal_smartled::{buffer_size_async, SmartLedsAdapterAsync};
use smart_leds::{colors, SmartLedsWriteAsync};

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) -> ! {
    // Initialize the HAL Peripherals
    let p = esp_hal::init(Config::default());
    let timg0 = TimerGroup::new(p.TIMG0);
    esp_hal_embassy::init(timg0.timer0);

    // Configure RMT (Remote Control Transceiver) peripheral globally
    // <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/peripherals/rmt.html>
    let rmt: Rmt<'_, esp_hal::Async> = {
        let frequency: Rate = {
            cfg_if::cfg_if! {
                if #[cfg(feature = "esp32h2")] {
                    Rate::from_mhz(32)
                } else {
                    Rate::from_mhz(80)
                }
            }
        };
        Rmt::new(p.RMT, frequency)
    }
    .expect("Failed to initialize RMT")
    .into_async();

    const NUM_LEDS: usize = 2;

    // We use one of the RMT channels to instantiate a `SmartLedsAdapterAsync` which can
    // be used directly with all `smart_led` implementations
    let rmt_channel = rmt.channel0;
    let rmt_buffer = [0_u32; buffer_size_async(NUM_LEDS)];

    // Each devkit uses a unique GPIO for the RGB LED, so in order to support
    // all chips we must unfortunately use `#[cfg]`s:
    let mut led = {
        cfg_if::cfg_if! {
            if #[cfg(feature = "esp32")] {
                SmartLedsAdapterAsync::new(rmt_channel, p.GPIO33, rmt_buffer)
            } else if #[cfg(feature = "esp32c3")] {
                SmartLedsAdapterAsync::new(rmt_channel, p.GPIO2, rmt_buffer)
            } else if #[cfg(any(feature = "esp32c6", feature = "esp32h2"))] {
                // hardcoded for the Waveshare ESP32-C6-LCD-1.9 development board.
                SmartLedsAdapterAsync::new(rmt_channel, p.GPIO3, rmt_buffer)
            } else if #[cfg(feature = "esp32s2")] {
                SmartLedsAdapterAsync::new(rmt_channel, p.GPIO18, rmt_buffer)
            } else if #[cfg(feature = "esp32s3")] {
                SmartLedsAdapterAsync::new(rmt_channel, p.GPIO48, rmt_buffer)
            }
        }
    };

    loop {
        let data = [colors::RED, colors::GREEN];
        led.write(data.into_iter()).await.unwrap();
        Timer::after(Duration::from_millis(10)).await;
    }
}
