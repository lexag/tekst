use crate::{handler::Handler, DISPLAY_HEIGHT, DISPLAY_WIDTH, POINT_SIZE};
use egui::{vec2, CentralPanel, Color32, Sense};
use tekst_common::textcontent::TextContent;

pub struct DesktopApp {
    handler: Handler,
    animation_start_time: f32,
}

impl Default for DesktopApp {
    fn default() -> Self {
        Self {
            handler: Handler::new(),
            animation_start_time: 0.0,
        }
    }
}

impl DesktopApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for DesktopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                self.handler
                    .tick(|buf| self.animation_start_time = ui.input(|i| i.time) as f32);

                //self.handler.renderer.egui_render(ui, POINT_SIZE);

                let (resp, p) = ui.allocate_painter(
                    vec2(
                        POINT_SIZE * DISPLAY_WIDTH as f32,
                        POINT_SIZE * DISPLAY_HEIGHT as f32,
                    ),
                    Sense::empty(),
                );

                p.rect_filled(resp.rect, 0.0, Color32::DARK_GRAY);

                for y in 0..DISPLAY_HEIGHT {
                    for x in 0..DISPLAY_WIDTH {
                        let is_red = self.handler.display.reds.bits[x / 8 + y * DISPLAY_WIDTH / 8]
                            & 0x1 << (7 - x % 8)
                            != 0;
                        let is_green = self.handler.display.greens.bits
                            [x / 8 + y * DISPLAY_WIDTH / 8]
                            & 0x1 << (7 - x % 8)
                            != 0;

                        let col = match (is_red, is_green) {
                            (true, true) => Color32::ORANGE,
                            (true, false) => Color32::RED,
                            (false, true) => Color32::GREEN,
                            (false, false) => Color32::BLACK,
                        };

                        let anim_timer = ui.input(|i| i.time) as f32 - self.animation_start_time;
                        let brightness_idx = ((75.0 * anim_timer
                            / self.handler.display.clock_divider as f32)
                            as usize)
                            .min(30)
                            .max(1);

                        let t = (255 - self.handler.display.brightnesses[brightness_idx]) as f32
                            / 255.0;

                        //println!(
                        //    "brightness: {} t: {} clkdiv: {}",
                        //    brightness_idx, t, self.handler.display.clock_divider
                        //);
                        p.circle_filled(
                            resp.rect.min
                                + vec2(POINT_SIZE * x as f32, POINT_SIZE * y as f32)
                                + vec2(POINT_SIZE, POINT_SIZE) * 0.5,
                            POINT_SIZE * 0.5,
                            col.lerp_to_gamma(Color32::BLACK, t),
                        );
                    }
                }
            });
        });
    }
}
