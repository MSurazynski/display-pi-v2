use std::thread::sleep;
use std::time::{Duration, Instant};

use rppal::gpio::{Gpio, InputPin, OutputPin};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

const RESET_GPIO_PIN: u8 = 17;
const DATA_OR_COMMAND_GPIO_PIN: u8 = 25;
const PANEL_POWER_GPIO_PIN: u8 = 18;
const BUSY_GPIO_PIN: u8 = 24;

const SPI_HZ: u32 = 4_000_000;
const SPI_MAX_TRANSFER: usize = 4096;
const BUSY_TIMEOUT: Duration = Duration::from_secs(60);
const BUSY_POLL_INTERVAL: Duration = Duration::from_millis(10);

pub const WIDTH: usize = 800;
pub const HEIGHT: usize = 480;
pub const BUFFER_SIZE: usize = WIDTH * HEIGHT / 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0x0,
    White = 0x1,
    Yellow = 0x2,
    Red = 0x3,
    Blue = 0x5,
    Green = 0x6,
}

#[derive(Debug, thiserror::Error)]
pub enum EpdError {
    #[error("gpio: {0}")]
    Gpio(#[from] rppal::gpio::Error),
    #[error("spi: {0}")]
    Spi(#[from] rppal::spi::Error),
    #[error("display stayed busy for too long")]
    BusyTimeout,
    #[error("image: {0}")]
    Image(#[from] image::ImageError),
    #[error("expected a 480x800 image, got {width}x{height}")]
    WrongImageSize { width: u32, height: u32 },
    #[error("pixel ({x}, {y}) has color {rgb:?}, which is not one of the 6 panel colors")]
    NotDithered { x: u32, y: u32, rgb: [u8; 3] },
}

impl Color {
    fn from_rgb(rgb: [u8; 3]) -> Option<Color> {
        match rgb {
            [0, 0, 0] => Some(Color::Black),
            [255, 255, 255] => Some(Color::White),
            [255, 255, 0] => Some(Color::Yellow),
            [255, 0, 0] => Some(Color::Red),
            [0, 0, 255] => Some(Color::Blue),
            [0, 255, 0] => Some(Color::Green),
            _ => None,
        }
    }
}

pub struct FrameBuffer {
    bytes: Box<[u8]>,
}

impl FrameBuffer {
    pub fn new(fill: Color) -> Self {
        let byte = ((fill as u8) << 4) | fill as u8;
        Self {
            bytes: vec![byte; BUFFER_SIZE].into_boxed_slice(),
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= WIDTH || y >= HEIGHT {
            return;
        }

        let index = y * (WIDTH / 2) + x / 2;
        let code = color as u8;
        let byte = &mut self.bytes[index];
        if x.is_multiple_of(2) {
            *byte = (*byte & 0x0F) | (code << 4);
        } else {
            *byte = (*byte & 0xF0) | code;
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn from_png(png_bytes: &[u8]) -> Result<Self, EpdError> {
        let portrait = image::load_from_memory(png_bytes)?.to_rgb8();

        if (portrait.width(), portrait.height()) != (HEIGHT as u32, WIDTH as u32) {
            return Err(EpdError::WrongImageSize {
                width: portrait.width(),
                height: portrait.height(),
            });
        }

        let mut frame_buffer = Self::new(Color::White);

        for (x, y, pixel) in portrait.enumerate_pixels() {
            let color =
                Color::from_rgb(pixel.0).ok_or(EpdError::NotDithered { x, y, rgb: pixel.0 })?;

            let panel_x = y as usize;
            let panel_y = (HEIGHT - 1) - x as usize;
            frame_buffer.set_pixel(panel_x, panel_y, color);
        }

        Ok(frame_buffer)
    }
}

pub struct Epd7in3e {
    spi_bus: Spi,
    reset_pin: OutputPin,
    data_or_command_pin: OutputPin,
    panel_power_pin: OutputPin,
    busy_pin: InputPin,
}

impl Epd7in3e {
    pub fn new() -> Result<Self, EpdError> {
        let gpio = Gpio::new()?;

        let reset_pin = gpio.get(RESET_GPIO_PIN)?.into_output_high();
        let data_or_command_pin = gpio.get(DATA_OR_COMMAND_GPIO_PIN)?.into_output_low();
        let panel_power_pin = gpio.get(PANEL_POWER_GPIO_PIN)?.into_output_high();
        let busy_pin = gpio.get(BUSY_GPIO_PIN)?.into_input();

        let spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, SPI_HZ, Mode::Mode0)?;

        Ok(Self {
            spi_bus: spi,
            reset_pin,
            data_or_command_pin,
            panel_power_pin,
            busy_pin,
        })
    }

    fn reset(&mut self) {
        self.reset_pin.set_high();
        sleep(Duration::from_millis(20));
        self.reset_pin.set_low();
        sleep(Duration::from_millis(2));
        self.reset_pin.set_high();
        sleep(Duration::from_millis(20));
    }

    fn send_command(&mut self, command: u8) -> Result<(), EpdError> {
        self.data_or_command_pin.set_low();
        self.spi_bus.write(&[command])?;
        Ok(())
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), EpdError> {
        self.data_or_command_pin.set_high();
        for chunk in data.chunks(SPI_MAX_TRANSFER) {
            self.spi_bus.write(chunk)?;
        }
        Ok(())
    }

    fn send_command_with_data(&mut self, command: u8, data: &[u8]) -> Result<(), EpdError> {
        self.send_command(command)?;
        if !data.is_empty() {
            self.send_data(data)?;
        }
        Ok(())
    }

    fn wait_until_idle(&mut self) -> Result<(), EpdError> {
        let now = Instant::now();
        while self.busy_pin.is_low() {
            if now.elapsed() > BUSY_TIMEOUT {
                return Err(EpdError::BusyTimeout);
            }
            sleep(BUSY_POLL_INTERVAL);
        }
        Ok(())
    }

    fn init(&mut self) -> Result<(), EpdError> {
        const INIT_SEQUENCE: &[(u8, &[u8])] = &[
            (0xAA, &[0x49, 0x55, 0x20, 0x08, 0x09, 0x18]),
            (0x01, &[0x3F]),
            (0x00, &[0x5F, 0x69]),
            (0x03, &[0x00, 0x54, 0x00, 0x44]),
            (0x05, &[0x40, 0x1F, 0x1F, 0x2C]),
            (0x06, &[0x6F, 0x1F, 0x17, 0x49]),
            (0x08, &[0x6F, 0x1F, 0x1F, 0x22]),
            (0x30, &[0x03]),
            (0x50, &[0x3F]),
            (0x60, &[0x02, 0x00]),
            (0x61, &[0x03, 0x20, 0x01, 0xE0]),
            (0x84, &[0x01]),
            (0xE3, &[0x2F]),
        ];

        self.reset();
        self.wait_until_idle()?;
        sleep(Duration::from_millis(30));

        for &(command, data) in INIT_SEQUENCE {
            self.send_command_with_data(command, data)?;
        }

        self.send_command(0x04)?;
        self.wait_until_idle()?;

        Ok(())
    }

    fn turn_on_display(&mut self) -> Result<(), EpdError> {
        self.send_command(0x04)?;
        self.wait_until_idle()?;

        self.send_command_with_data(0x06, &[0x6F, 0x1F, 0x17, 0x49])?;
        self.send_command_with_data(0x12, &[0x00])?;

        self.wait_until_idle()?;

        self.send_command_with_data(0x02, &[0x00])?;
        self.wait_until_idle()?;

        Ok(())
    }

    fn deep_sleep(&mut self) -> Result<(), EpdError> {
        self.send_command_with_data(0x02, &[0x00])?;
        self.wait_until_idle()?;
        self.send_command_with_data(0x07, &[0xA5])?;

        sleep(Duration::from_secs(2));
        Ok(())
    }

    pub fn display(&mut self, frame_buffer: &FrameBuffer) -> Result<(), EpdError> {
        self.init()?;
        self.send_command_with_data(0x10, frame_buffer.as_bytes())?;
        self.turn_on_display()?;
        self.deep_sleep()?;

        Ok(())
    }

    pub fn clear(&mut self, color: Color) -> Result<(), EpdError> {
        self.display(&FrameBuffer::new(color))?;
        Ok(())
    }
}

impl Drop for Epd7in3e {
    fn drop(&mut self) {
        self.data_or_command_pin.set_low();
        self.reset_pin.set_low();
        self.panel_power_pin.set_low();
    }
}
