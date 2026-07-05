use crate::renderer::DisplayBuffer;
use rppal::i2c;

pub struct SpiDriver {
    spi: i2c::I2c,
}

impl SpiDriver {
    pub fn new() -> Self {
	let mut i = i2c::I2c::new().unwrap();
	i.set_slave_address(0x30);
        Self {
            spi: i
        }
    }

    pub fn send_buffer(&mut self, img: DisplayBuffer) {
        self.spi.write(&img.brightnesses);
        self.spi.write(&img.reds.bits);
        self.spi.write(&img.greens.bits);
    }
}
