use crate::{cue::GlobalStyle, esds::Color};
use egui::{RichText, Widget};

use crate::DISPLAY_NUM_LINES;
use std::fmt::Display;

pub fn text_lines(
    cue: &crate::cue::Cue,
    ui: &mut egui::Ui,
    interactive: bool,
    global_style: GlobalStyle,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);
        for i in 0..DISPLAY_NUM_LINES {
            egui::TextEdit::singleline(&mut cue.text[i].clone())
                .interactive(interactive)
                .id_salt(ui.id().with(i))
                .horizontal_align(
                    cue.text_align
                        .unwrap_or(global_style.text_align)
                        .to_egui_align(),
                )
                .ui(ui);
        }
    });
}

pub fn property_with_default<T: Display>(ui: &mut egui::Ui, opt: Option<T>, default: &T) {
    if let Some(val) = opt {
        ui.monospace(val.to_string());
    } else {
        ui.add_enabled(false, egui::Label::new(default.to_string()));
    };
}

pub fn color_with_default(ui: &mut egui::Ui, opt: Option<Color>, default: &Color) {
    if let Some(val) = opt {
        ui.monospace(RichText::new(val.to_string()).color(val.to_egui_color()));
    } else {
        ui.add_enabled(
            false,
            egui::Label::new(RichText::new(default.to_string()).color(default.to_egui_color())),
        );
    };
}
