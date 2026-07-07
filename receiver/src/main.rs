const DISPLAY_WIDTH: usize = 448;
const DISPLAY_HEIGHT: usize = 32;

#[cfg(feature = "desktop")]
const POINT_SIZE: f32 = 3.0;

#[cfg(feature = "desktop")]
mod app;

#[cfg(feature = "embedded")]
mod spi;

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

#[cfg(feature = "embedded")]
fn main() {
    use crate::handler::Handler;
    use crate::spi;

    let mut handler = Handler::new();
    let mut spi = spi::SpiDriver::new();

    loop {
        use std::time::Duration;

        if let Some(display) = handler.tick() {
            spi.send_buffer(display);
        }
        std::thread::sleep(Duration::from_millis(100))
    }
}

fn init_filetree() {
    std::fs::create_dir_all("~/.tekst/imgs").unwrap();
}
