use crate::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
#[cfg(feature = "desktop")]
use egui::{vec2, Color32, Sense};
use serde::Deserialize;
use std::collections::HashMap;
use tekst_common::{
    primitive::{Color, TextAlign},
    textcontent::TextContent,
};

#[derive(Copy, Clone)]
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

impl DisplayBuffer {
    const LEFT: usize = 0;
    const RIGHT: usize = DISPLAY_WIDTH - 1;
    const MID_LR: usize = DISPLAY_WIDTH / 2;
    const TOP: usize = 0;
    const BOTTOM: usize = DISPLAY_HEIGHT - 1;
    const MID_TB: usize = DISPLAY_HEIGHT / 2;

    /// Solid color image
    pub fn test_pattern_a(col: Color, bright: u8) -> Self {
        let mut img = Self::new();
        if col.r() {
            img.reds.fill();
        }
        if col.g() {
            img.greens.fill();
        }
        img.brightnesses.fill(bright);
        img
    }

    /// Solid color image
    pub fn test_pattern_b() -> Self {
        let mut img = Self::new();
        img.reds.rect(Self::LEFT, Self::TOP, 5, 1);
        img.reds.rect(Self::LEFT, Self::TOP, 1, 5);
        img.reds.rect(Self::RIGHT - 5, Self::TOP, 5, 1);
        img.reds.rect(Self::RIGHT, Self::TOP, 1, 5);
        img.reds.rect(Self::LEFT, Self::BOTTOM, 5, 1);
        img.reds.rect(Self::LEFT, Self::BOTTOM - 4, 1, 5);
        img.reds.rect(Self::RIGHT - 4, Self::BOTTOM, 5, 1);
        img.reds.rect(Self::RIGHT, Self::BOTTOM - 4, 1, 5);

        img.reds.bits[BitBuffer::idx(Self::MID_LR, Self::MID_TB)] =
            ((DISPLAY_WIDTH & 0x00FF) as u8).reverse_bits();
        img.reds.bits[BitBuffer::idx(Self::MID_LR + 8, Self::MID_TB)] =
            (((DISPLAY_WIDTH & 0xFF00) >> 8) as u8).reverse_bits();
        img.reds.bits[BitBuffer::idx(Self::MID_LR, Self::MID_TB + 2)] =
            ((DISPLAY_HEIGHT & 0x00FF) as u8).reverse_bits();
        img.reds.bits[BitBuffer::idx(Self::MID_LR + 8, Self::MID_TB + 2)] =
            (((DISPLAY_HEIGHT & 0xFF00) >> 8) as u8).reverse_bits();
        img.greens.rect(Self::MID_LR - 5, Self::MID_TB - 1, 1, 5);

        img
    }
}

#[derive(Copy, Clone)]
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
    pub fn idx(x: usize, y: usize) -> usize {
        x / 8 + y * DISPLAY_WIDTH / 8
    }

    pub fn rect(&mut self, x: usize, y: usize, w: usize, h: usize) {
        //println!("{}, {}, {}, {}", x, y, h, w);

        for curs_y in y..y + h {
            let mut w_left = w;
            if w >= 8 {
                self.bits[Self::idx(x, curs_y)] |= 0xFF_u8 >> (x % 8);
            } else {
                self.bits[Self::idx(x, curs_y)] |= !(0xFF_u8 >> w as u8) >> (x as u8 % 8);
            }
            w_left -= (8 - x % 8).min(w_left);

            for curs_x in (x / 8 + 1)..(x / 8 + w_left / 8) {
                self.bits[Self::idx(curs_x, curs_y)] |= 0xFF_u8;
                w_left -= 8;
            }

            if w_left > 0 {
                self.bits[Self::idx(x + w + 8, curs_y)] |= !(0xFF_u8 >> w_left);
            }
        }
    }

    pub fn fill(&mut self) {
        self.bits.fill(0xFF);
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

#[derive(Deserialize, Copy, Clone, Debug)]
pub struct BoundingBox {
    x: usize,
    y: usize,
    w: usize,
    h: usize,
}

#[derive(Deserialize, Debug)]
pub struct BitGlyph {
    pub c: u32,
    pub bounding_box: BoundingBox,
    pub prepad: i32,
    pub postpad: i32,
    #[serde(skip)]
    pub rects: Vec<BoundingBox>,
}

pub struct TextRenderer {
    last_content: TextContent,
    glyphs: HashMap<char, BitGlyph>,
}

impl TextRenderer {
    pub fn new() -> Self {
        let bitmaps = [include_bytes!("../fonts/latin.bmp")];
        let glyphmaps = [include_bytes!("../fonts/latin.json")];

        let mut out_glyphs = HashMap::new();
        for (&bitmap, glyphmap) in bitmaps.iter().zip(glyphmaps) {
            let glyphs: Vec<BitGlyph> = serde_json::from_slice(glyphmap).unwrap();

            let bmp_start_offs = bitmap[10] as usize;

            const BMP_W: usize = 12 * 8;
            const BMP_H: usize = 13 * 16;

            for mut glyph in glyphs {
                let bmp_x = glyph.bounding_box.x;
                let bmp_y = BMP_H - glyph.bounding_box.y - 16;
                let start_idx = bmp_start_offs + (bmp_x + bmp_y * BMP_W) / 8;

                //println!(
                //    "char {}: bmp_coords: {}, {}\n start_idx: {}",
                //    char::from_u32(glyph.c).unwrap(),
                //    bmp_x,
                //    bmp_y,
                //    start_idx
                //);
                //println!("bb: {:#?}, BMP_W: {}, ", glyph.bounding_box, BMP_W);

                let stride = BMP_W / 8;
                let count = glyph.bounding_box.h;

                for y in 0..count {
                    let idx = start_idx + y * stride;
                    for bit in 0..8 {
                        if bitmap[idx] & 0x1 << (7 - bit) > 0 {
                            glyph.rects.push(BoundingBox {
                                x: bit,
                                y: 16 - y,
                                w: 1,
                                h: 1,
                            });
                            //print!("#");
                        } //else {
                          //print!(" ");
                          //}
                    }
                    //println!();
                }

                let Some(character) = char::from_u32(glyph.c) else {
                    continue;
                };
                out_glyphs.insert(character, glyph);
            }
            //println!("{:#?}", out_glyphs.get(&'A'));
            //println!("{:#?}", out_glyphs.get(&'A').unwrap().rects.len());
        }
        Self {
            glyphs: out_glyphs,
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

        self.metarender(self.last_content.clone(), |rect, color| {
            p.circle_filled(
                resp.rect.min + vec2(scale * rect.x as f32, scale * rect.y as f32),
                scale / 2.0,
                color.to_egui_color(),
            );
        });
    }

    pub fn render(&mut self, text: TextContent) -> DisplayBuffer {
        self.last_content = text.clone();

        let mut img = DisplayBuffer::new();

        self.metarender(text, |rect, color| {
            if color.r() {
                img.reds.rect(rect.x, rect.y, rect.w, rect.h);
            }
            if color.g() {
                img.greens.rect(rect.x, rect.y, rect.w, rect.h);
            }
        });
        img
    }

    pub fn metarender<F>(&mut self, text: TextContent, mut render_closure: F)
    where
        F: FnMut(BoundingBox, Color),
    {
        for (i, line) in text.text.iter().enumerate() {
            let glyphs = line.chars().filter_map(|c| self.glyphs.get(&c));

            let line_width: usize = glyphs
                .clone()
                .map(|g| g.bounding_box.w as i32 + g.postpad + g.prepad)
                .sum::<i32>() as usize;

            let mut horizontal_cursor = match text.align {
                TextAlign::Left => 0,
                TextAlign::Right => DISPLAY_WIDTH - line_width,
                TextAlign::Center => (DISPLAY_WIDTH / 2).saturating_sub(line_width / 2),
            };
            let vertical_cursor = DISPLAY_HEIGHT / text.text.len() * i;

            for glyph in glyphs {
                horizontal_cursor = (horizontal_cursor as i32 + glyph.prepad).max(0) as usize;
                for rect in &glyph.rects {
                    (render_closure)(
                        BoundingBox {
                            x: rect.x + horizontal_cursor,
                            y: rect.y + vertical_cursor - i,
                            ..*rect
                        },
                        text.color,
                    );
                }
                horizontal_cursor = (horizontal_cursor as i32 + glyph.postpad).max(0) as usize;
                horizontal_cursor += glyph.bounding_box.w;
            }
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
