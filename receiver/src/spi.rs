use crate::renderer::DisplayBuffer;
use rppal::i2c;

pub struct SpiDriver {
    spi: i2c::I2c,
}

impl SpiDriver {
    pub fn new() -> Self {
        let mut i = i2c::I2c::new().unwrap();
        i.set_slave_address(0x30);
        Self { spi: i }
    }

    pub fn send_buffer(&mut self, img: DisplayBuffer) {
        let mut brights = [0; 32];
        brights[1..].clone_from_slice(&img.brightnesses);
        brights[0] = img.clock_divider;
        self.spi.write(&brights);
        self.spi.write(&img.reds.bits);
        self.spi.write(&img.greens.bits);
    }
}
