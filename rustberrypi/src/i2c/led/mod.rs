use rppal::i2c::I2c;

use crate::errors::CommunicationError;

const HT16K33_BLINK_CMD: u8 = 0x80;
const HT16K33_BLINK_DISPLAYON: u8 = 0x01;
const HT16K33_SYSTEM_SETUP: u8 = 0x20;
const HT16K33_OSCILLATOR: u8 = 0x01;
const HT16K33_CMD_BRIGHTNESS: u8 = 0xE0;

pub enum Blink {
    Off = 0x00,
    HalfHz = 0x06,
    TwoHz = 0x04,
    OneHz = 0x02,
}

pub struct HT16K33 {
    device: I2c,
    buffer: [u8; 16],
}

impl HT16K33 {
    pub fn new(address: u8) -> Result<HT16K33, CommunicationError> {
        let mut device = I2c::new().unwrap();
        device
            .set_slave_address(address as u16)
            .map_err(|e| return CommunicationError::BusError(e))?;

        device
            .smbus_send_byte(HT16K33_SYSTEM_SETUP | HT16K33_OSCILLATOR)
            .map_err(|e| CommunicationError::BusError(e))?;

        let driver = HT16K33 {
            device,
            buffer: [0; 16],
        };
        driver.set_blink(Blink::Off)?;
        driver.set_brightness(15)?;

        Ok(driver)
    }

    pub fn set_blink(&self, frequency: Blink) -> Result<(), CommunicationError> {
        self.device
            .block_write(
                HT16K33_BLINK_CMD | HT16K33_BLINK_DISPLAYON | frequency as u8,
                &[],
            ).map_err(|e| CommunicationError::BusError(e))?;
        Ok(())
    }

    pub fn set_brightness(&self, brightness: u8) -> Result<(), CommunicationError> {
        if brightness >= 16 {
            panic!("Brightness can't be more than 15");
        }
        self.device
            .block_write(HT16K33_CMD_BRIGHTNESS | brightness, &[])
            .map_err(|e| CommunicationError::BusError(e))?;
        Ok(())
    }

    pub fn set_led(&mut self, led: u8, value: u8) -> Result<(), CommunicationError> {
        if led > 127 {
            panic!("LED must be between 0 and 127");
        }
        let pos: usize = led as usize / 8;
        println!("{}", pos);
        let offset = led % 8;
        if value == 0 {
            self.buffer[pos] &= !(1 << offset)
        } else {
            self.buffer[pos] |= 1 << offset
        }
        Ok(())
    }

    pub fn write_display(&self) -> Result<(), CommunicationError> {
        for (i, value) in self.buffer.iter().enumerate() {
            self.device
                .block_write(i as u8, &[*value])
                .map_err(|e| CommunicationError::BusError(e))?;
        }
        Ok(())
    }
}

#[derive(PartialEq)]
pub enum Color {
    Off = 0x00,
    Green = 0x01,
    Red = 0x02,
    Yellow = 0x03,
}

pub struct BicolorMatrix8x8 {
    controller: HT16K33,
}

impl BicolorMatrix8x8 {
    pub fn new() -> Result<Self, CommunicationError> {
        Ok(Self {
            controller: HT16K33::new(0x70)?,
        })
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, color: Color) -> Result<(), CommunicationError> {
        self.controller
            .set_led(y * 16 + x, if color == Color::Green { 1 } else { 0 })?;
        self.controller
            .set_led(y * 16 + x + 8, if color == Color::Red { 1 } else { 0 })?;
        Ok(())
    }

    pub fn write_display(&mut self) -> Result<(), CommunicationError> {
        self.controller.write_display()
    }
}