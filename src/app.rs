use crate::{
    autogo,
    cmdline::CommandLine,
    cue::{Cue, GlobalStyle},
    cuetable, elements,
    errorlog::ErrorLog,
    esds::TextAlign,
    hotkeys::{self, ShortcutMap, all_default_shortcuts},
    network::NetworkWriter,
    sequence::{Sequence, SequenceSlot},
    timecode::TimecodeReader,
};
use egui::{
    Align, Align2, Color32, FontId, Layout, Pos2, Rect, RichText, Sense, TextStyle, Widget,
};
use egui_file_dialog::FileDialog;
use oximedia::timecode::FrameRate;
use std::f32;

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
pub struct TekstApp {
    #[serde(skip)]
    pub file_dialog: FileDialog,
    #[serde(skip)]
    pub ctx: egui::Context,
    pub sequences: [Option<SequenceSlot>; 4],
    pub selected_sequence_idx: usize,
    pub patch_pointer: PatchPointer,
    pub file_pick_pointer: PatchPointer,
    pub global_style: GlobalStyle,
    #[serde(skip)]
    pub live_cue: Cue,
    pub default_cue: Cue,
    pub autoscroll: bool,
    #[serde(skip)]
    pub auto_follow: bool,
    #[serde(skip)]
    pub auto_timecode: bool,
    pub last_go_time: Option<f64>,
    #[serde(skip)]
    pub commandline: CommandLine,
    #[serde(skip)]
    pub network_writer: NetworkWriter,
    #[serde(skip)]
    pub error_log: ErrorLog,
    pub shortcuts: ShortcutMap,
    #[serde(skip)]
    pub timecode_reader: TimecodeReader,
}

impl Default for TekstApp {
    fn default() -> Self {
        Self {
            file_dialog: Default::default(),
            ctx: Default::default(),
            sequences: Default::default(),
            selected_sequence_idx: Default::default(),
            patch_pointer: Default::default(),
            file_pick_pointer: Default::default(),
            global_style: Default::default(),
            live_cue: Default::default(),
            default_cue: Default::default(),
            autoscroll: Default::default(),
            auto_follow: Default::default(),
            auto_timecode: Default::default(),
            shortcuts: all_default_shortcuts(),
            last_go_time: Default::default(),
            commandline: Default::default(),

            timecode_reader: TimecodeReader::new(),
            network_writer: NetworkWriter::default(),
            error_log: ErrorLog::new(),
        }
    }
}

const MATRIX_BUTTON_SIZE: (f32, f32) = (196.0, 96.0);

impl TekstApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let mut a: Self = if let Some(storage) = cc.storage
            && false
        {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            TekstApp::default()
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
        a.shortcuts.rebuild();
        a.reset_follow_time();
        a
    }

    pub fn selected_sequence(&mut self) -> &mut Option<SequenceSlot> {
        &mut self.sequences[self.selected_sequence_idx]
    }

    pub fn go_cue(&mut self, cue: &Cue) {
        if let Err(e) = self.network_writer.send_payload(&cue.make_payload()) {
            self.log_error(format!("Could not send on network: {e}"));
        }
        self.live_cue = cue.clone();
        self.reset_follow_time();
        self.ctx.request_repaint();
    }

    fn log_error(&mut self, message: String) {
        let time = self.ctx.input(|i| i.time);
        self.error_log.log(time, message);
    }

    pub fn reset_follow_time(&mut self) {
        self.last_go_time = Some(self.ctx.input(|i| i.time));
    }

    pub fn go(&mut self) {
        let new_cue = self.selected_cue();
        self.go_cue(&new_cue);
        self.swap_live_cue(new_cue);
    }

    fn swap_live_cue(&mut self, _new_cue: Cue) {
        match self.patch_pointer {
            PatchPointer::Sequence(..) => {
                if let Some(seq) = self.selected_sequence() {
                    seq.sequence.cue_pointer += 1;
                    seq.sequence.cue_pointer %= seq.sequence.cues.len();
                    self.autoscroll = true;
                }
            }
            _ => self.patch_pointer = PatchPointer::Blank,
        }
    }

    pub fn selected_cue(&self) -> Cue {
        match self.patch_pointer {
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
                if let PatchPointer::Sequence(seq_idx) = self.patch_pointer
                    && seq_idx == i
                {
                    true
                } else {
                    false
                },
            );

            if button_response.clicked() {
                self.patch_pointer = PatchPointer::Sequence(i);
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
            ui.checkbox(&mut self.autoscroll, "Autoscroll");
            ui.separator();
            ui.checkbox(&mut self.auto_follow, "AutoGo Follow");
            ui.separator();
            ui.checkbox(&mut self.auto_timecode, "AutoGo Timecode");
            ui.add(egui::Label::new(
                RichText::new(if let Some(time) = self.timecode_reader.timecode() {
                    time.to_string()
                } else {
                    "NO LTC FOUND".to_string()
                })
                .monospace()
                .color(if self.timecode_reader.confidence() > 0.8 {
                    Color32::GREEN
                } else {
                    Color32::ORANGE
                }),
            ))
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

        let base_color = self
            .selected_cue()
            .with_global_style(self.global_style)
            .text_color
            .unwrap_or_default()
            .to_egui_color();

        let mut color = base_color;

        if let Some(start) = self.last_go_time {
            let elapsed = now - start;

            if elapsed < 0.4 {
                #[allow(clippy::cast_possible_truncation)]
                let t = (elapsed / 0.4) as f32;

                // Pulse from bright back to normal
                color = Color32::WHITE.lerp_to_gamma(base_color, t);

                ui.ctx().request_repaint();
            }
        }

        egui::ProgressBar::new(1.0 - autogo::get_autogo_progress(self))
            .fill(color)
            .corner_radius(10.0)
            .ui(ui);
    }

    fn opaque_time(&self) -> f64 {
        self.ctx.input(|i| i.time)
    }

    fn error_label(&mut self, ui: &mut egui::Ui) {
        ui.add(
            #[allow(clippy::cast_possible_truncation)]
            egui::ProgressBar::new(self.error_log.countdown_progress() as f32)
                .fill(ui.visuals().error_fg_color)
                .corner_radius(0.0)
                .desired_height(5.0),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            self.error_log.update(self.opaque_time());
            if let Some(msg) = self.error_log.primary_error() {
                ui.colored_label(
                    ui.visuals().error_fg_color,
                    RichText::new(format!("{} (1/{})", msg, self.error_log.num_errors())).heading(),
                );
            } else {
                ui.heading("(0/0)");
            }
        });
    }

    fn command_line(&self, ui: &mut egui::Ui) {
        egui::TextEdit::singleline(&mut self.commandline.to_string())
            .interactive(false)
            .font(TextStyle::Heading)
            .desired_width(f32::INFINITY)
            .show(ui);
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
        autogo::handle_autogo_follow(self);
        self.error_log.update(self.opaque_time());
        if let Err(e) = self.timecode_reader.update() {
            self.log_error(format!("Timecode error {e}"))
        };

        ctx.request_repaint();

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
                ui.menu_button("Timecode", |ui| {
                    ui.menu_button("Framerate", |ui| {
                        for (name, fr) in [
                            ("23.976", FrameRate::Fps23976),
                            ("24", FrameRate::Fps24),
                            ("25", FrameRate::Fps25),
                            ("29.97 (DF)", FrameRate::Fps2997DF),
                            ("29.97 (NDF)", FrameRate::Fps2997NDF),
                            ("30", FrameRate::Fps30),
                        ] {
                            if ui
                                .selectable_label(self.timecode_reader.frame_rate() == fr, name)
                                .clicked()
                            {
                                self.timecode_reader.set_frame_rate(fr);
                            };
                        }
                    });
                    ui.menu_button("Input Device", |ui| {
                        if ui.button("Reload").clicked() {
                            self.timecode_reader.reload_available_devices();
                        }
                        ui.separator();
                        let devices = self.timecode_reader.available_devices().to_owned();
                        for (i, device) in devices.iter().enumerate() {
                            if ui
                                .selectable_label(
                                    self.timecode_reader.selected_device_idx() == Some(i),
                                    device,
                                )
                                .clicked()
                            {
                                self.timecode_reader.start(i);
                            }
                        }
                    });
                });

                elements::ip_address_entry(ui, &mut self.network_writer.config_mut().addr);
                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    powered_by_egui_and_eframe(ui);
                    egui::warn_if_debug_build(ui);
                });
            });
        });

        egui::TopBottomPanel::bottom("commandline").show(ctx, |ui| {
            self.command_line(ui);
            self.error_label(ui);
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
    fn text_align(_rect: Rect, idx: usize, align: TextAlign) -> Align2 {
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
