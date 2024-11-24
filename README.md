# SN3193 Driver for Rust
[![crates.io](https://img.shields.io/crates/v/sn3193.svg)](https://crates.io/crates/sn3193)
<!-- cargo-sync-readme start -->

This Rust `embedded-hal`-based library is a simple way to control the SN3193 RGB LED driver.
The SN3193 is a 3-channel LED driver with PWM control and breathing mode. It can drive up to
3 LEDs with a maximum current of 42 mA and is controlled via I2C. This driver is designed for the
`no_std` environment, so it can be used in embedded systems.
## Usage
Add this to your `Cargo.toml`:
```toml
[dependencies]
sn3193 = "0.1"
```
Then to create a new driver:
```rust
use embedded_hal::delay::DelayMs;
use embedded_hal::i2c::I2c;
use sn3193::SN3193Driver;

// board setup
let i2c = ...; // I2C peripheral
let delay = ...; // DelayMs implementation

// It is recommended that the `i2c` object be wrapped in an `embedded_hal_bus::i2c::CriticalSectionDevice` so that it can be shared between
// multiple peripherals.

// create and initialize the driver
let mut driver = SN3193Driver::new(i2c, delay);
if let Err(e) = driver.init() {
   panic!("Error initializing SN3193 driver: {:?}", e);
}
```
## Features
### PWM mode
The driver can set the LEDs to PWM mode. This allows you to set the brightness of each LED individually.
```rust
if let Err(e) = diver.set_led_mode(LEDModeSettings::PWM) {
   panic!("Error setting LED mode to PWM: {:?}", e);
}
// check your device wiring to know which LED is which
if let Err(e) = driver.set_pwm_levels(255, 128, 0) {
  panic!("Error setting PWM levels: {:?}", e);
}
```
### Breathing mode
The driver can set the LEDs to breathing mode. This mode allows you to set the time it takes for the LED to
ramp up to full brightness, hold at full brightness, ramp down to off, and hold at off. Each of these times
can be set individually for each LED. Furthermore, the PWM levels can be set for each LED.
```rust
// set the breathing times the same for all LEDs
if let Err(e) = driver.set_breathing_times_for_led(
    LEDId::ALL,
    BreathingIntroTime::Time1p04s,
    BreathingRampUpTime::Time4p16s,
    BreathingHoldHighTime::Time1p04s,
    BreathingRampDownTime::Time4p16s,
    BreathingHoldLowTime::Time2p08s,
) {
   panic!("Error setting breathing times: {:?}", e);
}
// enable breathing mode
if let Err(e) = driver.set_led_mode(LEDModeSettings::Breathing) {
  panic!("Error setting LED mode to breathing: {:?}", e);
}
```
The PWM levels and breathing times can be changed at any time. The driver will update the LEDs with the new settings.

### Function chaining
The driver functions return a `Result` that contains the driver reference in the `Ok` value. This
can be chained together to make the code more readable.
```rust
driver.set_led_mode(LEDModeSettings::PWM)?
    .set_current(CurrentSettings::Current17p5mA)?
    .set_pwm_levels(255, 128, 0)?
    .enable_leds(true, true, true)?;
```
## License
This library is licensed under the MIT license.

<!-- cargo-sync-readme end -->