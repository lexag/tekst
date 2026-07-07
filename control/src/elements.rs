use crate::cue::GlobalStyle;
use egui::{RichText, Widget};

use crate::DISPLAY_NUM_LINES;
use std::{
    fmt::Display,
    net::{Ipv4Addr, SocketAddrV4},
};
use tekst_common::primitive::Color;

pub fn text_lines(
    cue: &mut crate::cue::Cue,
    ui: &mut egui::Ui,
    interactive: bool,
    global_style: GlobalStyle,
) {
    ui.vertical(|ui| {
        ui.add_space(4.0);
        for i in 0..DISPLAY_NUM_LINES {
            egui::TextEdit::singleline(&mut cue.text[i])
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

pub fn ip_address_entry(ui: &mut egui::Ui, addr: &mut SocketAddrV4) {
    ui.label("IP:");
    let mut octets = addr.ip().octets();
    egui::DragValue::new(&mut octets[0]).ui(ui);
    egui::DragValue::new(&mut octets[1]).ui(ui);
    egui::DragValue::new(&mut octets[2]).ui(ui);
    egui::DragValue::new(&mut octets[3]).ui(ui);
    addr.set_ip(Ipv4Addr::from_octets(octets));
    ui.label("Port:");
    let mut port = addr.port();
    egui::DragValue::new(&mut port).ui(ui);
    addr.set_port(port);
}

pub fn slide_switch_selector<T: Display + PartialEq + Copy>(
    ui: &mut egui::Ui,
    val: &mut T,
    options: &[T],
) {
    egui::Frame::new()
        .fill(ui.visuals().code_bg_color)
        .show(ui, |ui| {
            for option in options {
                if ui
                    .selectable_label(*val == *option, option.to_string())
                    .clicked()
                {
                    *val = *option;
                }
            }
        });
}
