const DISPLAY_WIDTH: usize = 488;
const DISPLAY_HEIGHT: usize = 32;

#[cfg(feature = "desktop")]
const POINT_SIZE: f32 = 3.0;

#[cfg(feature = "desktop")]
mod app;

mod handler;
mod receiver;
mod renderer;

#[cfg(feature = "desktop")]
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([
                DISPLAY_WIDTH as f32 * POINT_SIZE + 3.0,
                DISPLAY_HEIGHT as f32 * POINT_SIZE + 20.0,
            ])
            .with_min_inner_size([
                DISPLAY_WIDTH as f32 * POINT_SIZE + 3.0,
                DISPLAY_HEIGHT as f32 * POINT_SIZE + 20.0,
            ]),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(app::DesktopApp::new(cc)))),
    )
}

#[cfg(not(feature = "desktop"))]
fn main() {}
