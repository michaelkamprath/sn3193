//! This Rust `embedded-hal`-based library is a simple way to control the SN3193 RGB LED driver.
//! The SN3193 is a 3-channel LED driver with PWM control and breathing mode. It can drive up to
//! 3 LEDs with a maximum current of 42 mA and is controlled via I2C. This driver is designed for the
//! `no_std` environment, so it can be used in embedded systems.
//! ## Usage
//! Add this to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! sn3193 = "0.1"
//! ```
//! Then to create a new driver:
//! ```rust
//! use embedded_hal::delay::DelayMs;
//! use embedded_hal::i2c::I2c;
//! use sn3193::{SN3193Driver;
//!
//! // board setup
//! let i2c = ...; // I2C peripheral
//! let delay = ...; // DelayMs implementation
//!
//! // It is recommended that the `i2c` object be wrapped in an `embedded_hal_bus::i2c::CriticalSectionDevice` so that it can be shared between
//! // multiple peripherals.
//!
//! // create and initialize the driver
//! let mut driver = SN3193Driver::new(i2c, delay);
//! if let Err(e) = driver.init() {
//!    panic!("Error initializing SN3193 driver: {:?}", e);
//! }
//! ```
//! ## Features
//! ### PWM mode
//! The driver can set the LEDs to PWM mode. This allows you to set the brightness of each LED individually.
//! ```rust
//! if let Err(e) = diver.set_led_mode(LEDModeSettings::PWM) {
//!    panic!("Error setting LED mode to PWM: {:?}", e);
//! }
//! // check your device wiring to know which LED is which
//! if let Err(e) = driver.set_pwm_levels(255, 128, 0) {
//!   panic!("Error setting PWM levels: {:?}", e);
//! }
//! ```
//! ### Breathing mode
//! The driver can set the LEDs to breathing mode. This mode allows you to set the time it takes for the LED to
//! ramp up to full brightness, hold at full brightness, ramp down to off, and hold at off. Each of these times
//! can be set individually for each LED. Furthermore, the PWM levels can be set for each LED.
//! ```rust
//! // set the breathing times the same for all LEDs
//! if let Err(e) = driver.set_breathing_times_for_led(
//!     LEDId::ALL,
//!     BreathingIntroTime::Time1p04s,
//!     BreathingRampUpTime::Time4p16s,
//!     BreathingHoldHighTime::Time1p04s,
//!     BreathingRampDownTime::Time4p16s,
//!     BreathingHoldLowTime::Time2p08s,
//! ) {
//!    panic!("Error setting breathing times: {:?}", e);
//! }
//! // enable breathing mode
//! if let Err(e) = driver.set_led_mode(LEDModeSettings::Breathing) {
//!   panic!("Error setting LED mode to breathing: {:?}", e);
//! }
//! ```
//! The PWM levels and breathing times can be changed at any time. The driver will update the LEDs with the new settings.
//!
//! ### Function chaining
//! The driver functions return a `Result` that contains the driver reference in the `Ok` value. This
//! can be chained together to make the code more readable.
//! ```rust
//! driver.set_led_mode(LEDModeSettings::PWM)?
//!     .set_current(CurrentSettings::Current17p5mA)?
//!     .set_pwm_levels(255, 128, 0)?
//!     .enable_leds(true, true, true)?;
//! ```
//! ## License
//! This library is licensed under the MIT license.

#![no_std]
#![allow(dead_code, clippy::unusual_byte_groupings)]
use embedded_hal::{delay::DelayNs, i2c};

// Registers
const REGISTER_SHUTDOWN: u8 = 0x00;
const REGISTER_BREATHING_CONTROL: u8 = 0x01;
const REGISTER_LED_MODE: u8 = 0x02;
const REGISTER_CURRENT_SETTING: u8 = 0x03;
const REGISTER_LED1_PWM: u8 = 0x04;
const REGISTER_LED2_PWM: u8 = 0x05;
const REGISTER_LED3_PWM: u8 = 0x06;
const REGISTER_DATA_UPDATE: u8 = 0x07;
const REGISTER_LED1_T0: u8 = 0x0A;
const REGISTER_LED2_T0: u8 = 0x0B;
const REGISTER_LED3_T0: u8 = 0x0C;
const REGISTER_LED1_T1T2: u8 = 0x10;
const REGISTER_LED2_T1T2: u8 = 0x11;
const REGISTER_LED3_T1T2: u8 = 0x12;
const REGISTER_LED1_T3T4: u8 = 0x16;
const REGISTER_LED2_T3T4: u8 = 0x17;
const REGISTER_LED3_T3T4: u8 = 0x18;
const REGISTER_TIME_UPDATE: u8 = 0x1C;
const REGISTER_LED_CONTROL: u8 = 0x1D;
const REGISTER_RESET: u8 = 0x2F;

// shutdown register
const SHUTDOWN_CHANNEL_ENABLE: u8 = 0b00_1_0000_0;
const SHUTDOWN_CHANNEL_DISABLE: u8 = 0b00_0_0000_0;
const SOFTWARE_SHUTDOWN_MODE: u8 = 0b00_0_0000_0;
const SOFTWARE_SHUTDOWN_NORMAL: u8 = 0b00_0_0000_1;

#[derive(Debug, PartialEq)]
pub enum CurrentSettings {
    Current42mA = 0b000_000_00,
    Current10mA = 0b000_001_00,
    Current5mA = 0b000_010_00,
    Current30mA = 0b000_011_00,
    Current17p5mA = 0b000_100_00,
}

/// LED mode settings
#[derive(Debug, PartialEq)]
pub enum LEDModeSettings {
    /// The LEDs are controlled by PWM settings
    PWM = 0b00_0_00000,
    /// The LEDs are controlled by breathing configuration
    Breathing = 0b00_1_00000,
}

/// The time it takes to start breathing.
/// The time is set in the T0 register.
#[derive(Debug, PartialEq)]
pub enum BreathingIntroTime {
    Time0s = 0b0000_0000,
    Time0p13s = 0b0001_0000,
    Time0p26s = 0b0010_0000,
    Time0p52s = 0b0011_0000,
    Time1p04s = 0b0100_0000,
    Time2p08s = 0b0101_0000,
    Time4p16s = 0b0110_0000,
    Time8p32s = 0b0111_0000,
    Time16p64s = 0b1000_0000,
    Time33p28s = 0b1001_0000,
    Time66p56s = 0b1010_0000,
}

/// The time it takes to ramp up from low to high brightness.
/// The time is set in the T1 register.
#[derive(Debug, PartialEq)]
pub enum BreathingRampUpTime {
    Time0p13s = 0b000_0000_0,
    Time0p26s = 0b001_0000_0,
    Time0p52s = 0b010_0000_0,
    Time1p04s = 0b011_0000_0,
    Time2p08s = 0b100_0000_0,
    Time4p16s = 0b101_0000_0,
    Time8p32s = 0b110_0000_0,
    Time16p64s = 0b111_0000_0,
}

/// The time that the high brightness is held.
/// The time is set in the T2 register.
#[derive(Debug, PartialEq)]
pub enum BreathingHoldHighTime {
    Time0s = 0b000_0000_0,
    Time0p13s = 0b000_0001_0,
    Time0p26s = 0b000_0010_0,
    Time0p52s = 0b000_0011_0,
    Time1p04s = 0b000_0100_0,
    Time2p08s = 0b000_0101_0,
    Time4p16s = 0b000_0110_0,
    Time8p32s = 0b000_0111_0,
    Time16p64s = 0b000_1000_0,
}

/// The time it takes to ramp down from high to low brightness.
/// The time is set in the T3 register.
#[derive(Debug, PartialEq)]
pub enum BreathingRampDownTime {
    Time0p13s = 0b000_0000_0,
    Time0p26s = 0b001_0000_0,
    Time0p52s = 0b010_0000_0,
    Time1p04s = 0b011_0000_0,
    Time2p08s = 0b100_0000_0,
    Time4p16s = 0b101_0000_0,
    Time8p32s = 0b110_0000_0,
    Time16p64s = 0b111_0000_0,
}

/// The time that the low brightness is held.
/// The time is set in the T4 register.
#[derive(Debug, PartialEq)]
pub enum BreathingHoldLowTime {
    Time0s = 0b000_0000_0,
    Time0p13s = 0b000_0001_0,
    Time0p26s = 0b000_0010_0,
    Time0p52s = 0b000_0011_0,
    Time1p04s = 0b000_0100_0,
    Time2p08s = 0b000_0101_0,
    Time4p16s = 0b000_0110_0,
    Time8p32s = 0b000_0111_0,
    Time16p64s = 0b000_1000_0,
    Time33p28s = 0b000_1001_0,
    Time66p56s = 0b000_1010_0,
}

#[derive(Debug, PartialEq)]
pub enum LEDId {
    LED1 = 0b001,
    LED2 = 0b010,
    LED3 = 0b100,
    ALL = 0b111,
}
/// Errors generated by the SN3193 driver
#[derive(Debug)]
pub enum SN3193Error<I2C>
where
    I2C: i2c::I2c,
{
    /// I2C bus error
    I2CError(I2C::Error),
}

#[cfg(feature = "defmt")]
impl<I2C> defmt::Format for SN3193Error<I2C>
where
    I2C: i2c::I2c,
{
    fn format(&self, fmt: defmt::Formatter) {
        let msg = match self {
            SN3193Error::I2CError(_) => "I2C error",
        };
        defmt::write!(fmt, "{}", msg);
    }
}

#[cfg(feature = "ufmt")]
impl<I2C> ufmt::uDisplay for CharacterDisplayError<I2C>
where
    I2C: i2c::I2c,
{
    fn fmt<W>(&self, w: &mut ufmt::Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: ufmt::uWrite + ?Sized,
    {
        let msg = match self {
            SN3193Error::I2CError(_) => "I2C error",
        };
        ufmt::uwrite!(w, "{}", msg)
    }
}

pub struct SN3193Driver<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    i2c: I2C,
    address: u8,
    delay: DELAY,
}

impl<I2C, DELAY> SN3193Driver<I2C, DELAY>
where
    I2C: i2c::I2c,
    DELAY: DelayNs,
{
    /// Default address for the SN3193. This is the address when the AD pin is connected to GND.
    pub fn default_address() -> u8 {
        0x68
    }

    /// Create a new SN3193 driver with the default address (0x68)
    pub fn new(i2c: I2C, delay: DELAY) -> Self {
        Self::new_with_address(i2c, delay, Self::default_address())
    }

    /// Create a new SN3193 driver with a specific address.
    /// The address can be changed by connecting the AD pin to GND, VDD, SCL, or SDA.
    /// The set of available addresses are:
    /// - 0x68 when AD is connected to GND
    /// - 0x6B when AD is connected to VDD
    /// - 0x69 when AD is connected to SCL
    /// - 0x6A when AD is connected to SDA
    pub fn new_with_address(i2c: I2C, delay: DELAY, address: u8) -> Self {
        Self {
            i2c,
            address,
            delay,
        }
    }

    /// Get a mutable reference to the I2C bus used by the driver
    pub fn i2c(&mut self) -> &mut I2C {
        &mut self.i2c
    }

    /// Initialize the SN3193 driver. This will set the LED mode to PWM, the current to 17.5 mA, and enable all LEDs.
    pub fn init(&mut self) -> Result<&mut Self, SN3193Error<I2C>> {
        // start up sequence
        // wait for power up
        self.delay.delay_ms(50);
        // reset
        self.i2c
            .write(self.address, &[REGISTER_RESET])
            .map_err(SN3193Error::I2CError)?;

        self.delay.delay_ms(50);
        self.i2c
            .write(
                self.address,
                &[
                    REGISTER_SHUTDOWN,
                    SHUTDOWN_CHANNEL_ENABLE | SOFTWARE_SHUTDOWN_MODE,
                ],
            )
            .map_err(SN3193Error::I2CError)?;

        // set mode 0 (PWM)
        self.set_led_mode(LEDModeSettings::PWM)?;

        // set current to 17.5 ma and enable all LEDs
        self.set_current(CurrentSettings::Current17p5mA)?
            .enable_leds(true, true, true)?;

        Ok(self)
    }

    /// Set the mode of the LED, either PWM or Breathing.
    pub fn set_led_mode(&mut self, mode: LEDModeSettings) -> Result<&mut Self, SN3193Error<I2C>> {
        // things seem to work better with a small delay here, but it's not in the datasheet
        self.delay.delay_ms(1);

        self.i2c
            .write(self.address, &[REGISTER_LED_MODE, mode as u8])
            .map_err(SN3193Error::I2CError)?;
        Ok(self)
    }

    /// Set the current for the LEDs.
    pub fn set_current(&mut self, current: CurrentSettings) -> Result<&mut Self, SN3193Error<I2C>> {
        // things seem to work better with a small delay here, but it's not in the datasheet
        self.delay.delay_ms(1);

        self.i2c
            .write(self.address, &[REGISTER_CURRENT_SETTING, current as u8])
            .map_err(SN3193Error::I2CError)?;
        Ok(self)
    }

    /// Enable or disable the LEDs. The LEDs are numbered 1, 2, and 3.
    pub fn enable_leds(
        &mut self,
        led1: bool,
        led2: bool,
        led3: bool,
    ) -> Result<&mut Self, SN3193Error<I2C>> {
        // things seem to work better with a small delay here, but it's not in the datasheet
        self.delay.delay_ms(1);

        let mut led_enable = 0;
        if led1 {
            led_enable |= 0b001;
        }
        if led2 {
            led_enable |= 0b010;
        }
        if led3 {
            led_enable |= 0b100;
        }
        self.i2c
            .write(self.address, &[REGISTER_LED_CONTROL, led_enable])
            .map_err(SN3193Error::I2CError)?;
        self.load_register_data()
    }

    /// Set the PWM levels for the RGB LED. 255 is full on, 0 is off.
    pub fn set_pwm_levels(
        &mut self,
        led1: u8,
        led2: u8,
        led3: u8,
    ) -> Result<&mut Self, SN3193Error<I2C>> {
        // things seem to work better with a small delay here, but it's not in the datasheet
        self.delay.delay_ms(1);
        self.i2c
            .write(self.address, &[REGISTER_LED1_PWM, led1])
            .map_err(SN3193Error::I2CError)?;
        // things seem to work better with a small delay here, but it's not in the datasheet
        self.delay.delay_ms(1);
        self.i2c
            .write(self.address, &[REGISTER_LED2_PWM, led2])
            .map_err(SN3193Error::I2CError)?;
        // things seem to work better with a small delay here, but it's not in the datasheet
        self.delay.delay_ms(1);
        self.i2c
            .write(self.address, &[REGISTER_LED3_PWM, led3])
            .map_err(SN3193Error::I2CError)?;
        self.load_register_data()
    }

    /// Set the breathing times for a particular LED. The times are set in the T0, T1, T2, T3, and T4 registers.
    /// The same values will be assigned to all LEDs if `LEDId::ALL` is used for the `led` parameter.
    pub fn set_breathing_times_for_led(
        &mut self,
        led: LEDId,
        intro: BreathingIntroTime,
        ramp_up: BreathingRampUpTime,
        hold_high: BreathingHoldHighTime,
        ramp_down: BreathingRampDownTime,
        hold_low: BreathingHoldLowTime,
    ) -> Result<&mut Self, SN3193Error<I2C>> {
        let t0_value = intro as u8;
        let t1t2_value = (ramp_up as u8) | (hold_high as u8);
        let t3t4_value = (ramp_down as u8) | (hold_low as u8);

        if led == LEDId::LED1 || led == LEDId::ALL {
            self.set_breathing_register(REGISTER_LED1_T0, t0_value)?;
            self.set_breathing_register(REGISTER_LED1_T1T2, t1t2_value)?;
            self.set_breathing_register(REGISTER_LED1_T3T4, t3t4_value)?;
        }
        if led == LEDId::LED2 || led == LEDId::ALL {
            self.set_breathing_register(REGISTER_LED2_T0, t0_value)?;
            self.set_breathing_register(REGISTER_LED2_T1T2, t1t2_value)?;
            self.set_breathing_register(REGISTER_LED2_T3T4, t3t4_value)?;
        }
        if led == LEDId::LED3 || led == LEDId::ALL {
            self.set_breathing_register(REGISTER_LED3_T0, t0_value)?;
            self.set_breathing_register(REGISTER_LED3_T1T2, t1t2_value)?;
            self.set_breathing_register(REGISTER_LED3_T3T4, t3t4_value)?;
        }
        self.load_register_time_data()?;
        Ok(self)
    }

    /// Load the register data. This is used to update the LEDs after changing the PWM levels. Private method.
    fn load_register_data(&mut self) -> Result<&mut Self, SN3193Error<I2C>> {
        // things seem to work better with a small delay here, but it's not in the datasheet
        self.delay.delay_ms(1);
        self.i2c
            .write(self.address, &[REGISTER_DATA_UPDATE, 0xFF])
            .map_err(SN3193Error::I2CError)?;
        Ok(self)
    }

    /// Load the breathing time data. This is used to update the LEDs after changing the breathing times. Private method.
    fn load_register_time_data(&mut self) -> Result<&mut Self, SN3193Error<I2C>> {
        // things seem to work better with a small delay here, but it's not in the datasheet
        self.delay.delay_ms(1);
        self.i2c
            .write(self.address, &[REGISTER_TIME_UPDATE, 0xFF])
            .map_err(SN3193Error::I2CError)?;
        Ok(self)
    }

    /// Set a breathing register. Private method.
    fn set_breathing_register(
        &mut self,
        register: u8,
        value: u8,
    ) -> Result<&mut Self, SN3193Error<I2C>> {
        // things seem to work better with a small delay here, but it's not in the datasheet
        self.delay.delay_ms(1);
        self.i2c
            .write(self.address, &[register, value])
            .map_err(SN3193Error::I2CError)?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use embedded_hal_mock::eh1::{
        delay::NoopDelay,
        i2c::{Mock as I2cMock, Transaction as I2cTransaction},
    };

    #[test]
    fn test_current_settings_into() {
        assert_eq!(CurrentSettings::Current42mA as u8, 0b000_000_00);
        assert_eq!(CurrentSettings::Current10mA as u8, 0b000_001_00);
        assert_eq!(CurrentSettings::Current5mA as u8, 0b000_010_00);
        assert_eq!(CurrentSettings::Current30mA as u8, 0b000_011_00);
        assert_eq!(CurrentSettings::Current17p5mA as u8, 0b000_100_00);
    }

    #[test]
    fn test_led_mode_settings_into() {
        assert_eq!(LEDModeSettings::PWM as u8, 0b00_0_00000);
        assert_eq!(LEDModeSettings::Breathing as u8, 0b00_1_00000);
    }

    #[test]
    fn test_set_led_mode() {
        let expectations = [I2cTransaction::write(
            0x68,
            std::vec![REGISTER_LED_MODE, LEDModeSettings::PWM as u8],
        )];
        let i2c = I2cMock::new(&expectations);
        let mut driver = SN3193Driver::new(i2c, NoopDelay);
        driver.set_led_mode(LEDModeSettings::PWM).unwrap();
        driver.i2c().done();
    }

    #[test]
    fn test_set_current() {
        let expectations = [
            I2cTransaction::write(
                0x68,
                std::vec![
                    REGISTER_CURRENT_SETTING,
                    CurrentSettings::Current17p5mA as u8
                ],
            ),
            I2cTransaction::write(
                0x68,
                std::vec![REGISTER_CURRENT_SETTING, CurrentSettings::Current42mA as u8],
            ),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut driver = SN3193Driver::new(i2c, NoopDelay);
        assert!(driver.set_current(CurrentSettings::Current17p5mA).is_ok());
        assert!(driver.set_current(CurrentSettings::Current42mA).is_ok());
        driver.i2c().done();
    }

    #[test]
    fn test_enable_leds() {
        let expectations = [
            I2cTransaction::write(0x68, std::vec![REGISTER_LED_CONTROL, 0b111]),
            I2cTransaction::write(0x68, std::vec![REGISTER_DATA_UPDATE, 0xFF]),
            I2cTransaction::write(0x68, std::vec![REGISTER_LED_CONTROL, 0b101]),
            I2cTransaction::write(0x68, std::vec![REGISTER_DATA_UPDATE, 0xFF]),
            I2cTransaction::write(0x68, std::vec![REGISTER_LED_CONTROL, 0b011]),
            I2cTransaction::write(0x68, std::vec![REGISTER_DATA_UPDATE, 0xFF]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut driver = SN3193Driver::new(i2c, NoopDelay);
        assert!(driver.enable_leds(true, true, true).is_ok());
        assert!(driver.enable_leds(true, false, true).is_ok());
        assert!(driver.enable_leds(true, true, false).is_ok());
        driver.i2c().done();
    }

    #[test]
    fn test_set_pwm_levels() {
        let expectations = [
            I2cTransaction::write(0x6B, std::vec![REGISTER_LED1_PWM, 255]),
            I2cTransaction::write(0x6B, std::vec![REGISTER_LED2_PWM, 128]),
            I2cTransaction::write(0x6B, std::vec![REGISTER_LED3_PWM, 0]),
            I2cTransaction::write(0x6B, std::vec![REGISTER_DATA_UPDATE, 0xFF]),
        ];
        let i2c = I2cMock::new(&expectations);
        let mut driver = SN3193Driver::new_with_address(i2c, NoopDelay, 0x6B);
        assert!(driver.set_pwm_levels(255, 128, 0).is_ok());
        driver.i2c().done();
    }
}
