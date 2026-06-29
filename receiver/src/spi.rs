use crate::renderer::DisplayBuffer;
use rppal::spi;

pub struct SpiDriver {
    spi: spi::Spi,
}

impl SpiDriver {
    const BUS: spi::Bus = spi::Bus::Spi0;

    pub fn new() -> Self {
        Self {
            spi: spi::Spi::new(
                Self::BUS,
                spi::SlaveSelect::Ss0,
                1_000_000_000,
                spi::Mode::Mode0,
            )
            .unwrap(),
        }
    }

    pub fn send_buffer(&mut self, img: DisplayBuffer) {
        self.spi.write(&img.brightnesses);
        self.spi.write(&img.reds.bits);
        self.spi.write(&img.greens.bits);
    }
}
