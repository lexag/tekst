use crate::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use cosmic_text::{Attrs, Color, FontFeatures, FontSystem, Metrics, Shaping, SwashCache};
#[cfg(feature = "desktop")]
use egui::{vec2, Color32, Sense};
use tekst_common::{primitive::TextAlign, textcontent::TextContent};

pub struct DisplayBuffer {
    pub brightnesses: [u8; DISPLAY_HEIGHT],

    // reds and greens are stored bitpacked row by row, with the first index on the bottom left like so:
    // bits : LSB         MSB LSB         MSB LSB         MSB etc.
    //        7 7 7 7 7 7 7 7 8 8 8 8 8 8 8 8 9 9 9 9 9 9 9 9 etc.
    // byte : 0 0 0 0 0 0 0 0 1 1 1 1 1 1 1 1 2 2 2 2 2 2 2 2
    //
    //
    pub reds: BitBuffer,
    pub greens: BitBuffer,
}

pub struct BitBuffer {
    pub bits: [u8; Self::BUFFER_SIZE],
}

impl Default for BitBuffer {
    fn default() -> Self {
        Self {
            bits: [0; Self::BUFFER_SIZE],
        }
    }
}

impl BitBuffer {
    const BUFFER_SIZE: usize = DISPLAY_WIDTH / 8 * DISPLAY_HEIGHT;

    fn new() -> Self {
        Self::default()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.bits.to_vec()
    }

    pub fn rect(&mut self, x: usize, y: usize, w: usize, h: usize) {
        println!("{}, {}, {}, {}", x, y, h, w);
        fn idx(x: usize, y: usize) -> usize {
            x / 8 + y * DISPLAY_WIDTH / 8
        }

        for curs_y in y..y + h {
            let mut w_left = w;
            if w >= 8 {
                self.bits[idx(x, curs_y)] |= 0xFF_u8 >> (x % 8);
            } else {
                self.bits[idx(x, curs_y)] |= !(0xFF_u8 >> w as u8) >> (x as u8 % 8);
            }
            w_left -= (8 - x % 8).min(w_left);

            for curs_x in (x / 8 + 1)..(x / 8 + w_left / 8) {
                self.bits[idx(curs_x, curs_y)] |= 0xFF_u8;
                w_left -= 8;
            }

            if w_left > 0 {
                self.bits[idx(x + w + 8, curs_y)] |= !(0xFF_u8 >> w_left);
            }
        }
    }
}

impl DisplayBuffer {
    pub fn new() -> Self {
        Self {
            brightnesses: [255_u8; DISPLAY_HEIGHT],
            reds: BitBuffer::new(),
            greens: BitBuffer::new(),
        }
    }
}

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    last_content: TextContent,
}

impl TextRenderer {
    pub fn new() -> Self {
        let mut font_system = FontSystem::new();
        println!(
            "num faces before load: {}",
            font_system.db().faces().count()
        );
        font_system
            .db_mut()
            .load_font_data(include_bytes!("../fonts/DotGothic16-Regular.ttf").to_vec());
        println!("num faces after load: {}", font_system.db().faces().count());

        for face in font_system.db().faces() {
            println!("family: {:?}", face.families);
        }

        Self {
            font_system,
            swash_cache: SwashCache::new(),
            last_content: TextContent::default(),
        }
    }

    #[cfg(feature = "desktop")]
    pub fn egui_render(&mut self, ui: &mut egui::Ui, scale: f32) {
        let (resp, p) = ui.allocate_painter(
            vec2(scale * DISPLAY_WIDTH as f32, scale * DISPLAY_HEIGHT as f32),
            Sense::empty(),
        );
        p.rect_filled(resp.rect, 0.0, Color32::DARK_BLUE);

        self.metarender(self.last_content.clone(), |x, y, w, h, color| {
            p.circle_filled(
                resp.rect.min + vec2(scale * x as f32, scale * y as f32),
                scale / 2.0,
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), color.a()),
            );
        });
    }

    pub fn render(&mut self, text: TextContent) -> DisplayBuffer {
        self.last_content = text.clone();

        let mut img = DisplayBuffer::new();

        const TRANSPARENCY_THRESHOLD: u8 = 140;
        self.metarender(text, |x, y, w, h, color| {
            if color.a() >= TRANSPARENCY_THRESHOLD {
                if color.r() > 0 {
                    img.reds
                        .rect(x as usize, y as usize, w as usize, h as usize);
                }
                if color.g() > 0 {
                    img.greens
                        .rect(x as usize, y as usize, w as usize, h as usize);
                }
            }
        });
        img
    }

    pub fn metarender<F>(&mut self, text: TextContent, mut render_closure: F)
    where
        F: FnMut(i32, i32, u32, u32, Color),
    {
        let size = text.font.size() as f32;
        let metrics = Metrics::new(16.0, 16.0);
        let attrs = Attrs::new().family(cosmic_text::Family::Name("DotGothic16"));

        let num_lines = text.text.len();
        for (i, line_text) in text.text.iter().enumerate() {
            let mut buffer = cosmic_text::Buffer::new(&mut self.font_system, metrics);
            let mut buffer = buffer.borrow_with(&mut self.font_system);

            buffer.set_size(
                Some(DISPLAY_WIDTH as f32),
                Some((DISPLAY_HEIGHT / num_lines) as f32),
            );
            buffer.set_text(
                line_text,
                &attrs,
                Shaping::Advanced,
                Some(text.align.to_cosmic_align()),
            );

            buffer.draw(
                &mut self.swash_cache,
                text.color.to_cosmic_color(),
                |x, y, w, h, color| {
                    (render_closure)(x, y + (DISPLAY_HEIGHT / num_lines * i) as i32, w, h, color)
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitbuf_rect() {
        let mut buf = BitBuffer::new();
        buf.rect(0, 0, 1, 1);
        assert_eq!(buf.to_vec()[0], 0b10000000);

        let mut buf = BitBuffer::new();
        buf.rect(0, 0, 8, 1);
        assert_eq!(buf.to_vec()[0], 0b11111111);
        assert_eq!(buf.to_vec()[1], 0b00000000);

        let mut buf = BitBuffer::new();
        buf.rect(0, 0, 9, 1);
        assert_eq!(buf.to_vec()[0], 0b11111111);
        assert_eq!(buf.to_vec()[1], 0b10000000);

        let mut buf = BitBuffer::new();
        buf.rect(0, 0, 9, 3);
        assert_eq!(buf.to_vec()[0], 0b11111111);
        assert_eq!(buf.to_vec()[1], 0b10000000);
        assert_eq!(buf.to_vec()[1 * DISPLAY_WIDTH / 8 + 0], 0b11111111);
        assert_eq!(buf.to_vec()[1 * DISPLAY_WIDTH / 8 + 1], 0b10000000);
        assert_eq!(buf.to_vec()[2 * DISPLAY_WIDTH / 8 + 0], 0b11111111);
        assert_eq!(buf.to_vec()[2 * DISPLAY_WIDTH / 8 + 1], 0b10000000);
    }
}
