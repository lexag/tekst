use crate::renderer::DisplayBuffer;
use rppal::i2c;

pub struct I2CDriver {
    i2c: i2c::I2c,
}

impl I2CDriver {
    pub fn new() -> Self {
        let mut i = i2c::I2c::new().unwrap();
        i.set_slave_address(0x30);
        Self { i2c: i }
    }

    pub fn send_buffer(&mut self, img: DisplayBuffer) {
        let mut brights = [0; 32];
        brights[1..].clone_from_slice(&img.brightnesses);
        brights[0] = img.clock_divider;
        self.i2c.write(&brights);
        self.i2c.write(&img.reds.bits);
        self.i2c.write(&img.greens.bits);
    }
}
