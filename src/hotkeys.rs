use crate::{
    app::{PatchPointer, TekstApp},
    cue::Cue,
};
use egui::{Key, KeyboardShortcut, Modifiers};
use egui_keybind::Shortcut;
use std::collections::HashMap;

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct ShortcutMap {
    pub actions: Vec<ActionID>,
    pub shortcuts: Vec<Shortcut>,
    #[serde(skip)]
    map: HashMap<ActionID, usize>,
}

impl ShortcutMap {
    pub fn new() -> Self {
        Self {
            actions: vec![],
            shortcuts: vec![],
            map: HashMap::new(),
        }
    }

    pub fn rebuild(&mut self) {
        for (i, k) in self.actions.iter().enumerate() {
            self.map.insert(k.clone(), i);
        }
    }

    pub fn add(&mut self, action_id: ActionID, shortcut: Shortcut) {
        self.actions.push(action_id.clone());
        self.shortcuts.push(shortcut);
        self.map.insert(action_id, self.actions.len() - 1);
    }

    pub fn get(&mut self, action_id: &ActionID) -> Option<Shortcut> {
        let idx = *self.map.get(action_id)?;
        self.shortcuts.get(idx).copied()
    }
}

#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Clone, Debug,
)]
pub enum ActionID {
    ChangePatch(PatchPointer),
    Go,
    SelectCueUp(usize),
    SelectCueDown(usize),
    SelectCueFirst,
    SelectCueLast,
    SelectCueChapterNext,
    SelectCueChapterPrev,
    CommandLineAppendToken(String),
    CommandLineAppendChar(String),
    CommandLinePlease,
    CommandLineBackspace,
    CommandLineCancel,
    GoCue(Cue),
}

pub fn exec_action(app: &mut TekstApp, action_id: ActionID) {
    match action_id {
        ActionID::ChangePatch(pointer) => {
            app.cue_pointer = pointer;
            if let PatchPointer::Sequence(i) = pointer {
                app.selected_sequence_idx = i
            }
        }
        ActionID::Go => app.go(),
        ActionID::SelectCueUp(num) => {
            if let Some(seq) = app.selected_sequence() {
                seq.sequence.cue_pointer = seq.sequence.cue_pointer.saturating_sub(num)
            }
        }
        ActionID::SelectCueDown(num) => {
            if let Some(seq) = app.selected_sequence() {
                seq.sequence.cue_pointer = seq.sequence.cue_pointer.saturating_add(num)
            }
        }
        ActionID::SelectCueFirst => {
            if let Some(seq) = app.selected_sequence() {
                seq.sequence.cue_pointer = 0
            }
        }
        ActionID::SelectCueLast => {
            if let Some(seq) = app.selected_sequence() {
                seq.sequence.cue_pointer = seq.sequence.cues.len() - 1
            }
        }
        ActionID::SelectCueChapterNext => {}
        ActionID::SelectCueChapterPrev => {}
        ActionID::CommandLineAppendToken(token) => {}
        ActionID::CommandLineAppendChar(c) => {}
        ActionID::CommandLinePlease => {}
        ActionID::CommandLineBackspace => {}
        ActionID::CommandLineCancel => {}
        ActionID::GoCue(cue) => app.go_cue(&cue),
    };
}

pub fn all_default_shortcuts() -> ShortcutMap {
    let mut shortcuts = ShortcutMap::new();

    add_patch_shortcuts(&mut shortcuts);
    add_commandline_shortcuts(&mut shortcuts);
    shortcuts.add(ActionID::Go, press(Key::Space));
    shortcuts.add(ActionID::SelectCueUp(1), press(Key::ArrowUp));
    shortcuts.add(ActionID::SelectCueDown(1), press(Key::ArrowDown));
    shortcuts.add(ActionID::GoCue(Cue::default()), press(Key::Delete));

    shortcuts
}

fn add_commandline_shortcuts(shortcuts: &mut ShortcutMap) -> Option<()> {
    for i in 0..10 {
        shortcuts.add(
            ActionID::CommandLineAppendChar(i.to_string()),
            press(Key::from_name(&i.to_string())?),
        );
    }

    shortcuts.add(ActionID::CommandLinePlease, press(Key::Enter));
    shortcuts.add(ActionID::CommandLineBackspace, press(Key::Backspace));
    shortcuts.add(ActionID::CommandLineCancel, press(Key::Escape));

    for (token, key) in [
        ("GOTO", Key::G),
        ("DELETE", Key::D),
        ("LOAD", Key::L),
        ("SEQ", Key::S),
        ("ART", Key::A),
        ("CUE", Key::Q),
        ("PATCH", Key::P),
        ("INSERT", Key::I),
        ("EDIT", Key::E),
        ("ALIGN", Key::X),
        ("COLOR", Key::C),
        ("FADE", Key::V),
        ("BRIGHTNESS", Key::B),
    ] {
        shortcuts.add(
            ActionID::CommandLineAppendToken(token.to_string()),
            press(key),
        );
    }

    None
}

fn add_patch_shortcuts(shortcuts: &mut ShortcutMap) {
    for (i, key) in [Key::F1, Key::F2, Key::F3, Key::F4].iter().enumerate() {
        shortcuts.add(
            ActionID::ChangePatch(PatchPointer::Sequence(i)),
            press(*key),
        );
    }
    for (i, key) in [Key::F5, Key::F6, Key::F7, Key::F8].iter().enumerate() {
        shortcuts.add(
            ActionID::ChangePatch(PatchPointer::PatchCue(i)),
            press(*key),
        );
    }
    for (i, key) in [Key::F9, Key::F10, Key::F11, Key::F12].iter().enumerate() {
        shortcuts.add(
            ActionID::ChangePatch(PatchPointer::PatchImageCue(i)),
            press(*key),
        );
    }
}

fn press(key: Key) -> Shortcut {
    Shortcut::new(Some(KeyboardShortcut::new(Modifiers::NONE, key)), None)
}
fn ctrl(key: Key) -> Shortcut {
    Shortcut::new(Some(KeyboardShortcut::new(Modifiers::CTRL, key)), None)
}
fn shift(key: Key) -> Shortcut {
    Shortcut::new(Some(KeyboardShortcut::new(Modifiers::SHIFT, key)), None)
}
