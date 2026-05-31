use crate::{
    cue::{Cue, GlobalStyle},
    cuetable,
    esds::TextAlign,
    hotkeys::{self, ShortcutMap},
    sequence::{Sequence, SequenceSlot},
};
use egui::{Align, Align2, Color32, FontId, Pos2, Rect, Sense, Widget, vec2};
use egui_file_dialog::FileDialog;
use egui_table::Table;
use std::time::{Duration, Instant};

#[derive(
    serde::Deserialize,
    serde::Serialize,
    Default,
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Ord,
    Eq,
    Hash,
    Copy,
)]
pub enum PatchPointer {
    #[default]
    Blank,
    Sequence(usize),
    PatchCue(usize),
    PatchImageCue(usize),
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
#[derive(Default)]
pub struct TekstApp {
    #[serde(skip)]
    pub file_dialog: FileDialog,
    #[serde(skip)]
    pub ctx: egui::Context,
    pub sequences: [Option<SequenceSlot>; 4],
    pub selected_sequence_idx: usize,
    pub cue_pointer: PatchPointer,
    pub file_pick_pointer: PatchPointer,
    pub global_style: GlobalStyle,
    #[serde(skip)]
    pub live_cue: Cue,
    pub default_cue: Cue,
    pub autoscroll: bool,
    pub shortcuts: ShortcutMap,
    pub last_go_time: Option<f64>,
}

const MATRIX_BUTTON_SIZE: (f32, f32) = (196.0, 96.0);

impl TekstApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let mut a: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        a.ctx = cc.egui_ctx.clone();
        a.file_dialog = FileDialog::new();
        for sequence in &mut a.sequences {
            if let Some(seq) = sequence.as_mut() {
                *seq = SequenceSlot::load_from_path(seq.path.clone()).unwrap_or_default();
            }
        }
        a.live_cue = Cue {
            text: [
                "Hello".to_string(),
                "Testing a really long message".to_string(),
            ],
            ..Default::default()
        };
        a.shortcuts = hotkeys::all_default_shortcuts();
        a
    }

    pub fn selected_sequence(&mut self) -> &mut Option<SequenceSlot> {
        &mut self.sequences[self.selected_sequence_idx]
    }

    pub fn send_payload(&mut self, payload: Vec<u8>) {}

    pub fn go_cue(&mut self, cue: &Cue) {
        self.send_payload(cue.make_payload());
        self.live_cue = cue.clone();
        self.last_go_time = Some(self.ctx.input(|i| i.time));
        self.ctx.request_repaint();
    }

    pub fn go(&mut self) {
        let new_cue = self.selected_cue();
        self.go_cue(&new_cue);
        self.swap_live_cue(new_cue);
    }

    fn swap_live_cue(&mut self, new_cue: Cue) {
        match self.cue_pointer {
            PatchPointer::Sequence(..) => {
                if let Some(seq) = self.selected_sequence() {
                    seq.sequence.cue_pointer += 1;
                    seq.sequence.cue_pointer %= seq.sequence.cues.len();
                    self.autoscroll = true;
                }
            }
            _ => self.cue_pointer = PatchPointer::Blank,
        }
    }

    pub fn selected_cue(&self) -> Cue {
        match self.cue_pointer {
            PatchPointer::Sequence(idx) => {
                let sequence = &self.sequences[idx];
                if let Some(seq) = sequence {
                    &seq.sequence.cues[seq.sequence.cue_pointer]
                } else {
                    &self.default_cue
                }
            }
            _ => &self.default_cue,
        }
        .clone()
        .with_global_style(self.global_style)
    }

    pub fn load_sequence_file(&mut self, sequence_idx: usize) {
        self.file_pick_pointer = PatchPointer::Sequence(sequence_idx);
        self.file_dialog.pick_file();
    }

    fn sequence_button(&mut self, ui: &mut egui::Ui, i: usize) {
        let sequence = &self.sequences[i];
        let button_response = if let Some(seq) = sequence {
            let button_response = matrix_button(
                ui,
                &seq.sequence.name,
                if let PatchPointer::Sequence(seq_idx) = self.cue_pointer
                    && seq_idx == i
                {
                    true
                } else {
                    false
                },
            );

            if button_response.clicked() {
                self.cue_pointer = PatchPointer::Sequence(i);
                self.selected_sequence_idx = i;
            }
            button_response
        } else {
            let msg: &str = "LOAD SEQ";
            matrix_button(ui, msg, false)
        };
        if button_response.secondary_clicked() {
            self.load_sequence_file(i)
        }
    }

    fn sequences_matrix(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sequences");
        ui.horizontal(|ui| {
            for i in 0..self.sequences.len() {
                self.sequence_button(ui, i);
            }
        });
    }

    fn follow_settings_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Autoscroll");
            ui.checkbox(&mut self.autoscroll, "");
        });
    }
    fn global_settings_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Global Brightness:");
            egui::DragValue::new(&mut self.global_style.brightness).ui(ui);
            ui.label("Global Fade:");
            egui::DragValue::new(&mut self.global_style.fade_speed)
                .range(0..=9)
                .speed(0.02)
                .ui(ui);
            ui.label("Global Font:");
            egui::DragValue::new(&mut self.global_style.text_font).ui(ui);
            ui.label("Global Color:");
            self.global_style.text_color.ui_selector(ui);
            ui.label("Global Align:");
            self.global_style.text_align.ui_selector(ui);
        });
    }

    pub fn handle_keybinds(&mut self) {
        let keys = self.shortcuts.actions.clone();
        for action in keys {
            let shortcut = self.shortcuts.get(&action);

            if let Some(shortcut) = shortcut
                && let Some(kbd) = shortcut.keyboard()
                && !self.ctx.wants_keyboard_input()
                && self
                    .ctx
                    .input(|i| i.modifiers == kbd.modifiers && i.key_pressed(kbd.logical_key))
            {
                hotkeys::exec_action(self, action);
            }
        }
    }

    fn go_flasher(&mut self, ui: &mut egui::Ui) {
        let now = ui.ctx().input(|i| i.time);

        let col = self
            .selected_cue()
            .with_global_style(self.global_style)
            .text_color
            .unwrap_or_default()
            .to_egui_color();

        let base_r = col.r();
        let base_g = col.g();
        let base_b = col.b();

        let mut color = egui::Color32::from_rgb(base_r, base_g, base_b);

        if let Some(start) = self.last_go_time {
            let elapsed = now - start;

            if elapsed < 0.4 {
                let t = (elapsed / 0.4) as f32;

                // Pulse from bright back to normal
                color = egui::Color32::from_rgb(
                    (255.0 * (1.0 - t) + base_r as f32 * t) as u8,
                    (255.0 * (1.0 - t) + base_g as f32 * t) as u8,
                    (255.0 * (1.0 - t) + base_b as f32 * t) as u8,
                );

                ui.ctx().request_repaint();
            } else {
                self.last_go_time = None;
            }
        }

        egui::ProgressBar::new(1.0)
            .fill(color)
            .corner_radius(10.0)
            .ui(ui);
    }
}

fn matrix_button(ui: &mut egui::Ui, msg: &str, selected: bool) -> egui::Response {
    ui.add(
        egui::Button::new(msg)
            .wrap()
            .min_size(MATRIX_BUTTON_SIZE.into())
            .selected(selected),
    )
}

impl eframe::App for TekstApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.file_dialog.update(ctx);
        self.handle_keybinds();

        if let Some(path) = self.file_dialog.take_picked() {
            match self.file_pick_pointer {
                PatchPointer::Sequence(sequence_idx) => {
                    self.sequences[sequence_idx] = SequenceSlot::load_from_path(path);
                }
                _ => {}
            }
        }

        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Write demo sequence file").clicked() {
                        SequenceSlot {
                            path: "".into(),
                            sequence: Sequence::example(),
                        }
                        .save_to_path(
                            std::env::current_dir()
                                .unwrap_or_default()
                                .join("example_sequence.csv"),
                        );
                    }
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    powered_by_egui_and_eframe(ui);
                    egui::warn_if_debug_build(ui);
                });
            });
        });

        egui::SidePanel::right("cuetable")
            .resizable(false)
            .exact_width(960.0)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    self.follow_settings_bar(ui);
                    ui.separator();
                    self.global_settings_bar(ui);
                    ui.separator();
                    cuetable::cue_table(self, ui);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.take_available_space();
            ui.vertical(|ui| {
                ui.vertical(|ui| {
                    ui.heading("DISPLAY PROGRAM");
                    render_screen_preview(ui, &self.live_cue, false);
                    self.go_flasher(ui);
                    ui.heading("DISPLAY PREVIEW");
                    render_screen_preview(ui, &self.selected_cue(), true);
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.heading("Toolbar");
                    ui.horizontal(|ui| {
                        if matrix_button(ui, "GO", false).clicked() {
                            self.go();
                        }
                    });
                    ui.separator();
                    self.sequences_matrix(ui);
                });
            });
        });
    }
}

fn render_screen_preview(ui: &mut egui::Ui, cue: &Cue, peek_brightness: bool) {
    fn text_anchor(rect: Rect, idx: usize, align: TextAlign) -> Pos2 {
        const EDGE_SPACING_VERTICAL: f32 = 5.0;
        const EDGE_SPACING_HORIZONTAL: f32 = 10.0;
        let y = if idx == 0 {
            rect.top() + EDGE_SPACING_VERTICAL
        } else {
            rect.bottom() - EDGE_SPACING_VERTICAL
        };
        let x = match align {
            TextAlign::Left => rect.left() + EDGE_SPACING_HORIZONTAL,
            TextAlign::Right => rect.right() - EDGE_SPACING_HORIZONTAL,
            TextAlign::Center => rect.center().x,
        };

        (x, y).into()
    }
    fn text_align(rect: Rect, idx: usize, align: TextAlign) -> Align2 {
        let vert = if idx == 0 { Align::Min } else { Align::Max };
        let hor = match align {
            TextAlign::Left => Align::Min,
            TextAlign::Right => Align::Max,
            TextAlign::Center => Align::Center,
        };

        Align2((hor, vert).into())
    }

    let (resp, p) = ui.allocate_painter((ui.available_width(), 128.0).into(), Sense::CLICK);
    p.rect_filled(resp.rect, 10.0, Color32::BLACK);
    let align = cue.text_align.unwrap_or_default();
    let brightness_factor = cue.brightness.unwrap_or_default() as f32 / 255.0;
    for idx in [0, 1] {
        let anchor = text_anchor(resp.rect, idx, align);
        let alignment = text_align(resp.rect, idx, align);
        let text = cue.text[idx].clone();
        let font_id = FontId::new(48.0, egui::FontFamily::Proportional);
        p.text(
            anchor,
            alignment,
            &text,
            font_id.clone(),
            cue.text_color
                .unwrap_or_default()
                .to_egui_color()
                .gamma_multiply(brightness_factor),
        );
        if peek_brightness {
            p.text(
                anchor,
                alignment,
                text,
                font_id,
                Color32::DARK_GRAY.gamma_multiply(
                    (0.5 - cue.brightness.unwrap_or_default() as f32 / 255.0).max(0.0),
                ),
            );
        }
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by egui and eframe.");
    });
}
