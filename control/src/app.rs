use crate::{
    autogo::{self, AutoGo, AutoGoConsolidator, AutoGoOpMode, AutoTimecode},
    cmdline::CommandLine,
    cue::{Cue, GlobalStyle},
    cuetable, elements,
    errorlog::ErrorLog,
    hotkeys::{self, ShortcutMap, all_default_shortcuts},
    network::NetworkWriter,
    sequence::{Sequence, SequenceSlot},
};
use egui::{
    Align, Align2, Color32, Context, FontId, Layout, Pos2, Rect, RichText, Sense, Stroke,
    StrokeKind, TextStyle, Widget,
};
use egui_file_dialog::FileDialog;
use ks_common_ui::{
    autoenum::InlineWidgetAutoEnum,
    component_interface::{ConfigurationWidget, InlineWidget, InlineWidgetMenu},
    style,
};
use std::{f32, fmt::Display};
use tekst_common::{
    primitive::TextAlign,
    protocol::{DisplayContent, Message},
    textcontent::TextContent,
};

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

#[derive(serde::Deserialize, serde::Serialize, Default, Debug, Copy, Clone, PartialEq)]
pub enum OpMode {
    #[default]
    Demo,
    Live,
    Edit,
}

impl Display for OpMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpMode::Demo => write!(f, "Demo"),
            OpMode::Live => write!(f, "Live"),
            OpMode::Edit => write!(f, "Edit"),
        }
    }
}

impl InlineWidgetAutoEnum for OpMode {
    fn options() -> Vec<Self>
    where
        Self: Sized + Display,
    {
        vec![Self::Demo, Self::Live, Self::Edit]
    }

    fn color(&self) -> Option<Color32> {
        match self {
            OpMode::Demo => Some(ks_common_ui::style::WARNING_COLOR),
            OpMode::Live => Some(ks_common_ui::style::ACCENT_COLOR),
            OpMode::Edit => Some(ks_common_ui::style::WARNING_COLOR),
        }
    }

    fn text(&self) -> Option<String> {
        Some(self.to_string().to_uppercase())
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TekstApp {
    #[serde(skip)]
    pub file_dialog: FileDialog,
    #[serde(skip)]
    pub ctx: egui::Context,
    pub sequences: [Option<SequenceSlot>; 12],
    pub selected_sequence_idx: usize,
    pub file_pick_pointer: PatchPointer,
    pub global_style: GlobalStyle,
    pub display_font_size: f32,
    #[serde(skip)]
    pub live_content: TextContent,
    pub back_content: TextContent,
    pub default_cue: Cue,
    pub autoscroll: bool,
    pub op_mode: OpMode,
    pub last_go_time: Option<f64>,
    #[serde(skip)]
    pub commandline: CommandLine,
    pub network_writer: NetworkWriter,
    #[serde(skip)]
    pub error_log: ErrorLog,
    pub shortcuts: ShortcutMap,
    #[serde(skip)]
    pub autogo: AutoGoConsolidator,
}

impl Default for TekstApp {
    fn default() -> Self {
        let ctx = Context::default();
        Self {
            back_content: TextContent::default(),
            file_dialog: Default::default(),
            ctx: ctx.clone(),
            display_font_size: 48.0,
            sequences: Default::default(),
            selected_sequence_idx: Default::default(),
            file_pick_pointer: Default::default(),
            global_style: Default::default(),
            live_content: Default::default(),
            default_cue: Default::default(),
            autoscroll: Default::default(),
            shortcuts: all_default_shortcuts(),
            last_go_time: Default::default(),
            commandline: Default::default(),
            op_mode: OpMode::Demo,

            autogo: AutoGoConsolidator::new(ctx.clone()),
            network_writer: NetworkWriter::default(),
            error_log: ErrorLog::new(),
        }
    }
}

const MATRIX_BUTTON_SIZE: (f32, f32) = (224.0, 64.0);

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
            TekstApp::default()
        };
        a.ctx = cc.egui_ctx.clone();
        ks_common_ui::style::load_fonts(&mut a.ctx);
        a.file_dialog = FileDialog::new();
        a.shortcuts = all_default_shortcuts();
        let mut errs_to_log = vec![];
        for sequence in &mut a.sequences {
            if let Some(seq) = sequence.as_mut() {
                let res = SequenceSlot::load_from_path(seq.path.clone());
                match res {
                    Ok(s) => {
                        *seq = if s.sequence.cues.is_empty() {
                            SequenceSlot {
                                path: s.path,
                                sequence: Sequence::example(),
                            }
                        } else {
                            s
                        }
                    }
                    Err(e) => {
                        errs_to_log.push(format!("{}", e));
                        *seq = SequenceSlot {
                            path: seq.path.clone(),
                            sequence: Sequence::example(),
                        }
                    }
                }
            }
        }

        for err in errs_to_log {
            a.log_error(err);
        }
        a.autogo = AutoGoConsolidator::new(a.ctx.clone());
        a.live_content = TextContent {
            text: [
                "Hello".to_string(),
                "Testing a really long message".to_string(),
            ]
            .to_vec(),
            ..Default::default()
        };
        a.shortcuts.rebuild();
        a.reset_follow_time();
        a
    }

    pub fn selected_sequence_mut(&mut self) -> &mut Option<SequenceSlot> {
        &mut self.sequences[self.selected_sequence_idx]
    }

    pub fn selected_sequence(&self) -> Option<&SequenceSlot> {
        self.sequences[self.selected_sequence_idx].as_ref()
    }

    pub fn go_cue(&mut self, cue: &Cue, learn: bool) {
        if self.op_mode == OpMode::Live {
            self.try_send_payload(cue.clone().with_global_style(self.global_style));
        }

        if learn {
            let learned_cue = self.autogo.go_happened(self.selected_cue().clone());
            *self.selected_cue_mut() = learned_cue;
        }

        if self.live_content.text.iter().any(|s| !s.is_empty()) {
            self.back_content = self.live_content.clone();
        }
        self.live_content = cue.clone().with_global_style(self.global_style);
        self.reset_follow_time();
        self.ctx.request_repaint();
    }

    fn try_send_payload(&mut self, content: TextContent) {
        let json = match serde_json::to_string(&Message::Show(DisplayContent::Text(content))) {
            Ok(val) => val,
            Err(e) => {
                self.log_error(format!("Could not serialize text content: {e}"));
                return;
            }
        };
        let res = self.network_writer.send_payload(json.as_bytes());
        if let Err(e) = res {
            self.log_error(format!("Could not send on network: {e}"));
        }
    }

    fn log_error(&mut self, message: String) {
        let time = self.ctx.input(|i| i.time);
        self.error_log.log(time, message);
    }

    pub fn reset_follow_time(&mut self) {
        self.last_go_time = Some(self.ctx.input(|i| i.time));
    }

    pub fn go(&mut self) {
        let new_cue = self.selected_cue().clone();
        self.go_cue(&new_cue, true);
        self.swap_live_cue(new_cue.clone());
    }

    pub fn go_back(&mut self) {
        if self.op_mode == OpMode::Live {
            self.try_send_payload(self.back_content.clone());
        }
        self.live_content = self.back_content.clone();
    }

    fn swap_live_cue(&mut self, _new_cue: Cue) {
        if let Some(seq) = self.selected_sequence_mut() {
            seq.sequence.cue_pointer += 1;
            seq.sequence.cue_pointer %= seq.sequence.cues.len();
            self.autoscroll = true;
        }
    }

    pub fn selected_cue_mut(&mut self) -> &mut Cue {
        let sequence = self.sequences[self.selected_sequence_idx].as_mut();
        if let Some(seq) = sequence {
            &mut seq.sequence.cues[seq.sequence.cue_pointer]
        } else {
            &mut self.default_cue
        }
    }

    pub fn selected_cue_with_global(&self) -> TextContent {
        self.selected_cue()
            .clone()
            .with_global_style(self.global_style)
    }

    pub fn selected_cue(&self) -> &Cue {
        let sequence = &self.sequences[self.selected_sequence_idx];
        if let Some(seq) = sequence {
            &seq.sequence.cues[seq.sequence.cue_pointer]
        } else {
            &self.default_cue
        }
    }

    pub fn load_sequence_file(&mut self, sequence_idx: usize) -> bool {
        if sequence_idx >= self.sequences.len() {
            return false;
        };
        self.file_pick_pointer = PatchPointer::Sequence(sequence_idx);
        self.file_dialog.pick_file();
        true
    }

    pub fn save_sequence(&mut self, sequence_idx: usize) {
        if let Some(seq) = &self.sequences[sequence_idx] {
            if let Err(e) = seq.save_to_path(seq.path.clone()) {
                self.log_error(format!("{}", e));
            };
        }
    }

    fn sequence_button(&mut self, ui: &mut egui::Ui, i: usize) {
        ui.vertical(|ui| {
            let sequence = &self.sequences[i];
            let cue = if let Some(seq) = sequence {
                seq.sequence.cues[seq.sequence.cue_pointer].clone()
            } else {
                Cue::default()
            };
            render_screen_preview(
                ui,
                &cue.clone().with_global_style(self.global_style),
                true,
                12.0,
                MATRIX_BUTTON_SIZE.0,
                cue.autogo_timecode.is_some() || cue.autogo_delay_ms.is_some(),
            );

            let button_response = if let Some(seq) = sequence {
                let button_response = matrix_button(
                    ui,
                    &seq.sequence.name,
                    self.selected_sequence_idx == i,
                    (i + 1).to_string(),
                );

                if button_response.clicked() {
                    self.selected_sequence_idx = i;
                }
                button_response
            } else {
                let msg: &str = "LOAD SEQ";
                matrix_button(ui, msg, false, (i + 1).to_string())
            };
            if button_response.secondary_clicked() {
                self.load_sequence_file(i);
            }
        });
    }

    fn sequences_matrix(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sequences");
        ui.vertical(|ui| {
            for i in 0..3 {
                ui.horizontal_wrapped(|ui| {
                    for j in i * 4..(i + 1) * 4 {
                        self.sequence_button(ui, j);
                    }
                });
            }
        });
    }

    fn global_settings_bar(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("global-settings-bar").show(ui, |ui| {
            ui.monospace("SCROLL");
            ui.monospace("BRIGHT");
            ui.monospace("FADE");
            ui.monospace("FONT");
            ui.monospace("COLOR");
            ui.monospace("ALIGN");
            ui.end_row();

            self.autoscroll.inline_widget(ui);
            self.global_style
                .brightness
                .clone()
                .inline_widget_menu(ui, |ui| {
                    self.global_style.brightness.draw_configuration(ui);
                });

            self.global_style.fade_speed.autoenum_inline_widget_menu(ui);
            self.global_style.text_font.autoenum_inline_widget_menu(ui);
            self.global_style.text_color.autoenum_inline_widget_menu(ui);
            self.global_style.text_align.autoenum_inline_widget_menu(ui);
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

        let base_color = ks_common_ui::style::ACCENT_COLOR;

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

        egui::ProgressBar::new(1.0 - self.autogo.progress(&self.selected_cue()))
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

fn matrix_button(
    ui: &mut egui::Ui,
    msg: &str,
    selected: bool,
    help_text: String,
) -> egui::Response {
    ui.add(
        egui::Button::new(msg)
            .wrap()
            .min_size(MATRIX_BUTTON_SIZE.into())
            .shortcut_text(help_text)
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
        ctx.set_style(ks_common_ui::style::style());

        self.file_dialog.update(ctx);
        self.handle_keybinds();
        if self.autogo.requests_go(&self.selected_cue()) {
            self.go();
        }
        self.error_log.update(self.opaque_time());

        // FIXME: this should be handled and bubbled up in the autogo function stack, updating
        // timecode reader when we want info
        if let Err(e) = self.autogo.timecode.timecode_reader.update() {
            self.log_error(format!("Timecode error {e}"))
        };

        ctx.request_repaint();

        if let Some(path) = self.file_dialog.take_picked() {
            match self.file_pick_pointer {
                PatchPointer::Sequence(sequence_idx) => {
                    let res = SequenceSlot::load_from_path(path);
                    match res {
                        Ok(s) => self.sequences[sequence_idx] = Some(s),
                        Err(e) => self.log_error(format!("{}", e)),
                    }
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
                    if ui.button("Save all sequences").clicked() {
                        for i in 0..self.sequences.len() {
                            self.save_sequence(i);
                        }
                    }
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

                    //if ui.button("Send [.] blanking message").clicked() {
                    //    self.network_writer
                    //        .send_payload(&Cue::default().make_payload_with_data(vec![b'.']));
                    //}
                    //if ui.button("Send [ ] blanking message").clicked() {
                    //    self.network_writer
                    //        .send_payload(&Cue::default().make_payload_with_data(vec![]));
                    //}
                    if ui.button("Quit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                egui::Grid::new("upper_statusbar_grid").show(ui, |ui| {
                    ui.monospace("LTC INPUT");
                    ui.monospace("LTC DELAY");
                    ui.monospace("FOLLOW TIME");
                    ui.monospace("DISPLAY IP");
                    ui.monospace("AFW");
                    ui.monospace("ATC");
                    ui.monospace("MODE");

                    ui.end_row();

                    self.autogo
                        .timecode
                        .timecode_reader
                        .timecode()
                        .inline_widget_menu(ui, |ui| {
                            self.autogo.timecode.timecode_reader.draw_configuration(ui);
                        });
                    self.autogo
                        .timecode
                        .offset
                        .clone()
                        .inline_widget_menu(ui, |ui| {
                            self.autogo.timecode.offset.draw_configuration(ui);
                        });

                    (self.autogo.follow.elapsed().min(999.999) as f32).inline_widget(ui);

                    self.network_writer
                        .config_mut()
                        .addr
                        .clone()
                        .inline_widget_menu(ui, |ui| {
                            self.network_writer.config_mut().addr.draw_configuration(ui);
                        });

                    self.autogo
                        .follow
                        .mode_mut()
                        .autoenum_inline_widget_menu(ui);
                    self.autogo
                        .timecode
                        .mode_mut()
                        .autoenum_inline_widget_menu(ui);
                    self.op_mode.autoenum_inline_widget_menu(ui);
                });

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
            .exact_width(ctx.content_rect().width() / 2.0)
            .show(ctx, |ui| {
                self.global_settings_bar(ui);
                ui.separator();
                cuetable::cue_table(self, ui);
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.take_available_space();
            ui.vertical(|ui| {
                ui.vertical(|ui| {
                    ui.heading("DISPLAY PROGRAM");
                    render_screen_preview(
                        ui,
                        &self.live_content,
                        false,
                        self.display_font_size,
                        950.0,
                        false,
                    );
                    self.go_flasher(ui);
                    ui.heading("DISPLAY PREVIEW");
                    render_screen_preview(
                        ui,
                        &self.selected_cue_with_global(),
                        true,
                        self.display_font_size,
                        950.0,
                        false,
                    );
                    ui.separator();
                    ui.heading("Description");
                    egui::TextEdit::multiline(&mut self.selected_cue().description.clone())
                        .interactive(false)
                        .desired_width(f32::INFINITY)
                        .ui(ui);
                });
                ui.separator();
                ui.vertical(|ui| {
                    self.sequences_matrix(ui);
                });
            });
        });
    }
}

fn render_screen_preview(
    ui: &mut egui::Ui,
    content: &TextContent,
    peek_brightness: bool,
    font_size: f32,
    width: f32,
    border: bool,
) {
    fn text_anchor(rect: Rect, idx: usize, align: TextAlign, font_size: f32) -> Pos2 {
        const EDGE_SPACING_HORIZONTAL: f32 = 10.0;
        let edge_spacing_vertical: f32 = font_size * 0.1;
        let y = if idx == 0 {
            rect.top() + edge_spacing_vertical
        } else {
            rect.bottom() - edge_spacing_vertical
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

    let (resp, p) = ui.allocate_painter((width, font_size * 2.5).into(), Sense::CLICK);
    p.rect_filled(resp.rect, 10.0, Color32::BLACK);
    let align = content.align;
    let brightness_factor = content.brightness as f32 / 255.0;
    for idx in [0, 1] {
        let anchor = text_anchor(resp.rect, idx, align, font_size);
        let alignment = text_align(resp.rect, idx, align);
        let text = content.text[idx].clone();
        let font_id = FontId::new(font_size, egui::FontFamily::Proportional);
        p.text(
            anchor,
            alignment,
            &text,
            font_id.clone(),
            content
                .color
                .to_egui_color()
                .gamma_multiply(brightness_factor),
        );
        if peek_brightness {
            p.text(
                anchor,
                alignment,
                text,
                font_id,
                Color32::DARK_GRAY
                    .gamma_multiply((0.5 - content.brightness as f32 / 255.0).max(0.0)),
            );
        }
    }

    if border {
        p.rect_stroke(
            resp.rect,
            10.0,
            Stroke::new(2.0, style::ACTIVE_COLOR),
            StrokeKind::Inside,
        );
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by egui and eframe.");
    });
}
