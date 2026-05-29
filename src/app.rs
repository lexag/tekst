use crate::{
    DISPLAY_NUM_LINES,
    cue::{Cue, GlobalStyle},
    sequence::{Sequence, SequenceSlot},
};
use egui::{Align2, Color32, FontId, RichText, Sense, Stroke, Widget, pos2, vec2};
use egui_file_dialog::FileDialog;

#[derive(serde::Deserialize, serde::Serialize, Default)]
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
    file_dialog: FileDialog,
    #[serde(skip)]
    ctx: egui::Context,
    sequences: [Option<SequenceSlot>; 4],
    selected_sequence_idx: usize,
    cue_pointer: PatchPointer,
    file_pick_pointer: PatchPointer,
    global_style: GlobalStyle,
    #[serde(skip)]
    live_cue: Cue,
    default_cue: Cue,
}

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
        a
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
}

impl eframe::App for TekstApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.file_dialog.update(ui.ctx());

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

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
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
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
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

        egui::Panel::left("left_panel")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.take_available_space();
                ui.vertical(|ui| {
                    ui.vertical(|ui| {
                        ui.heading("DISPLAY PROGRAM");
                        render_screen_preview(ui, &self.live_cue);
                        ui.heading("DISPLAY PREVIEW");
                        render_screen_preview(ui, &self.selected_cue());
                    });

                    ui.vertical(|ui| {
                        ui.heading("Sequences");
                        ui.horizontal(|ui| {
                            for i in 0..self.sequences.len() {
                                let sequence = &self.sequences[i];
                                let button_response = if let Some(seq) = sequence {
                                    let button_response = ui.add(
                                        egui::Button::new(seq.sequence.name.clone())
                                            .wrap()
                                            .min_size((256.0, 96.0).into()),
                                    );

                                    if button_response.clicked() {
                                        self.cue_pointer = PatchPointer::Sequence(i)
                                    }
                                    button_response
                                } else {
                                    ui.add(
                                        egui::Button::new("LOAD SEQ")
                                            .wrap()
                                            .min_size((256.0, 96.0).into()),
                                    )
                                };
                                if button_response.secondary_clicked() {
                                    self.load_sequence_file(i)
                                }
                            }
                        });
                    });
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
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
                ui.separator();
                egui_table::Table::new()
                    .headers([egui_table::HeaderRow {
                        height: 32.0,
                        groups: vec![(0..2), (2..4), (4..7), (7..9)],
                    }])
                    .num_sticky_cols(2)
                    .columns([
                        egui_table::Column::new(64.0).resizable(false),
                        egui_table::Column::new(256.0).resizable(false),
                        egui_table::Column::new(64.0).resizable(false),
                        egui_table::Column::new(64.0).resizable(false),
                        egui_table::Column::new(64.0).resizable(false),
                        egui_table::Column::new(64.0).resizable(false),
                        egui_table::Column::new(64.0).resizable(false),
                        egui_table::Column::new(128.0).resizable(false),
                        egui_table::Column::new(128.0).resizable(false),
                    ])
                    .num_rows(
                        if let Some(seq) = &self.sequences[self.selected_sequence_idx] {
                            seq.sequence.cues.len().try_into().unwrap_or_default()
                        } else {
                            1
                        },
                    )
                    .show(
                        ui,
                        &mut ScriptLineListDelegate {
                            sequence: &mut self.sequences[self.selected_sequence_idx],
                            global_style: self.global_style,
                        },
                    );
            });
        });
    }
}

fn render_screen_preview(ui: &mut egui::Ui, cue: &Cue) {
    let (resp, p) = ui.allocate_painter((ui.available_width(), 128.0).into(), Sense::CLICK);
    p.rect_filled(resp.rect, 10.0, Color32::BLACK);
    p.text(
        resp.rect.center_top() + vec2(0.0, 5.0),
        Align2::CENTER_TOP,
        cue.text[0].clone(),
        FontId::new(48.0, egui::FontFamily::Proportional),
        cue.text_color
            .unwrap_or_default()
            .to_egui_color()
            .gamma_multiply(cue.brightness.unwrap_or_default() as f32 / 255.0),
    );
    p.text(
        resp.rect.center_bottom() + vec2(0.0, -5.0),
        Align2::CENTER_BOTTOM,
        cue.text[1].clone(),
        FontId::new(48.0, egui::FontFamily::Proportional),
        cue.text_color
            .unwrap_or_default()
            .to_egui_color()
            .gamma_multiply(cue.brightness.unwrap_or_default() as f32 / 255.0),
    );
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by egui and eframe.");
    });
}

struct ScriptLineListDelegate<'a> {
    sequence: &'a mut Option<SequenceSlot>,
    global_style: GlobalStyle,
}

// headers:
// ident
// text
// brightness
// fade speed (if overridden)
// color
// align
// font
// follow delay
// timecode follow timestamp
// auto follow visual countdown

impl egui_table::TableDelegate for ScriptLineListDelegate<'_> {
    fn row_ui(&mut self, ui: &mut egui::Ui, row_nr: u64) {
        if row_nr.is_multiple_of(2) {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
    }

    fn default_row_height(&self) -> f32 {
        16.0 + 16.0 * DISPLAY_NUM_LINES as f32
    }

    fn header_cell_ui(&mut self, ui: &mut egui::Ui, cell: &egui_table::HeaderCellInfo) {
        match cell.col_range.start {
            0 => ui.label("Line"),
            2 => ui.label("Brightness / Fade"),
            4 => ui.label("Design"),
            7 => ui.label("Auto Follow"),
            _ => ui.label("N/A"),
        };
    }

    fn cell_ui(&mut self, ui: &mut egui::Ui, cell: &egui_table::CellInfo) {
        let col_nr = cell.col_nr;
        let row_nr = cell.row_nr;

        let rect = ui.max_rect();
        let x = rect.right();
        let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
        ui.painter()
            .line_segment([pos2(x, rect.top()), pos2(x, rect.bottom())], stroke);
        ui.painter()
            .rect_stroke(rect, 2.0, stroke, egui::StrokeKind::Inside);

        if let Some(seq) = self.sequence {
            if row_nr as usize == seq.sequence.cue_pointer {
                ui.painter().rect_stroke(
                    rect,
                    2.0,
                    ui.visuals().widgets.noninteractive.fg_stroke,
                    egui::StrokeKind::Inside,
                );
            }
            let cue = &seq.sequence.cues[row_nr as usize];
            ui.centered_and_justified(|ui| {
                match col_nr {
                    0 => {
                        if ui.monospace(cue.ident.clone()).clicked() {
                            seq.sequence.cue_pointer = row_nr as usize;
                        }
                    }
                    1 => {
                        ui.vertical(|ui| {
                            ui.add_space(4.0);
                            for i in 0..DISPLAY_NUM_LINES {
                                egui::TextEdit::singleline(&mut cue.text[i].clone())
                                    .interactive(false)
                                    .id_salt(i + 0x354634 + row_nr as usize)
                                    .horizontal_align(egui::Align::Center)
                                    .ui(ui);
                            }
                        });
                    }
                    2 => {
                        if let Some(val) = cue.brightness {
                            ui.monospace(val.to_string());
                        } else {
                            ui.add_enabled(
                                false,
                                egui::Label::new(self.global_style.brightness.to_string()),
                            );
                        };
                    }
                    3 => {
                        if let Some(val) = cue.fade_speed {
                            ui.monospace(val.to_string());
                        } else {
                            ui.add_enabled(
                                false,
                                egui::Label::new(self.global_style.fade_speed.to_string()),
                            );
                        };
                    }
                    4 => {
                        if let Some(val) = cue.text_color {
                            ui.add(egui::Label::new(
                                RichText::new(val.to_string())
                                    .monospace()
                                    .color(val.to_egui_color()),
                            ));
                        } else {
                            ui.add_enabled(
                                false,
                                egui::Label::new(self.global_style.text_color.to_string()),
                            );
                        };
                    }
                    5 => {
                        if let Some(val) = cue.text_align {
                            ui.add(egui::Label::new(RichText::new(val.to_string()).monospace()));
                        } else {
                            ui.add_enabled(
                                false,
                                egui::Label::new(self.global_style.text_align.to_string()),
                            );
                        };
                    }
                    6 => {
                        if let Some(val) = cue.text_font {
                            ui.monospace(val.to_string());
                        } else {
                            ui.add_enabled(
                                false,
                                egui::Label::new(self.global_style.text_font.to_string()),
                            );
                        };
                    }
                    7 => {
                        if let Some(val) = cue.autogo_delay_ms {
                            ui.monospace(format!("{:.3} s", val as f32 / 1000.0));
                        };
                    }
                    8 => {
                        if let Some(val) = cue.autogo_timecode {
                            ui.monospace(val.to_string());
                        };
                    }
                    _ => {
                        ui.label("N/A");
                    }
                };
            });
            if (row_nr as usize) < seq.sequence.cue_pointer {
                ui.painter()
                    .rect_filled(rect, 0.0, ui.visuals().panel_fill.gamma_multiply(0.5));
            }
        }
    }
}
