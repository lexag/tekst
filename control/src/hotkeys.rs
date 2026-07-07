use crate::{
    app::{PatchPointer, TekstApp},
    autogo::{AutoGo, AutoGoOpMode},
    cmdline::CommandLineToken,
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

const CMDLINE_HOTKEYS: [(CommandLineToken, Key); 16] = [
    (CommandLineToken::Goto, Key::G),
    (CommandLineToken::Delete, Key::D),
    (CommandLineToken::Seq, Key::S),
    (CommandLineToken::Cue, Key::Q),
    (CommandLineToken::To, Key::H),
    (CommandLineToken::Insert, Key::I),
    (CommandLineToken::Append, Key::A),
    (CommandLineToken::Split, Key::Y),
    (CommandLineToken::Merge, Key::U),
    (CommandLineToken::Edit, Key::E),
    (CommandLineToken::Parent, Key::P),
    (CommandLineToken::Align, Key::X),
    (CommandLineToken::Color, Key::C),
    (CommandLineToken::Time, Key::Z),
    (CommandLineToken::Transition, Key::V),
    (CommandLineToken::Brightness, Key::B),
];

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

    pub fn get(&self, action_id: &ActionID) -> Option<Shortcut> {
        let idx = *self.map.get(action_id)?;
        self.shortcuts.get(idx).copied()
    }
}

#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Clone, Debug,
)]
pub enum ActionID {
    ChangeSequence(usize),
    Go,
    SelectCueUp(usize),
    SelectCueDown(usize),
    SelectCueFirst,
    SelectCueLast,
    SelectCueMarkNext,
    SelectCueMarkPrev,
    CommandLineAppendToken(CommandLineToken),
    CommandLineAppendChar(char),
    CommandLinePlease,
    CommandLineBackspace,
    CommandLineCancel,
    ToggleAutoscroll,
    ToggleAutoGo,
    GoCue(Cue),
}

#[allow(clippy::too_many_lines)]
pub fn exec_action(app: &mut TekstApp, action_id: ActionID) {
    match action_id {
        ActionID::ChangeSequence(pointer) => {
            app.selected_sequence_idx = pointer;
            app.autogo.dry_go_happened();
        }
        ActionID::Go => app.go(),
        ActionID::SelectCueUp(num) => {
            try_cue_sub(app, num);
            app.autogo.dry_go_happened();
        }
        ActionID::SelectCueDown(num) => {
            try_cue_add(app, num);
            app.autogo.dry_go_happened();
        }
        ActionID::SelectCueFirst => {
            try_cue_first(app);
            app.autogo.dry_go_happened();
        }
        ActionID::SelectCueLast => {
            try_cue_last(app);
            app.autogo.dry_go_happened();
        }
        ActionID::SelectCueMarkNext => {
            try_cue_mark_next(app);
            app.autogo.dry_go_happened();
        }
        ActionID::SelectCueMarkPrev => {
            try_cue_mark_prev(app);
            app.autogo.dry_go_happened();
        }
        ActionID::CommandLineAppendToken(token) => {
            app.commandline.push_token(token);
        }
        ActionID::CommandLineAppendChar(c) => {
            app.commandline.push_char(c);
        }
        ActionID::CommandLinePlease => {
            let cmd = app.commandline.clone();
            cmd.execute(app);
            app.commandline.clear();
        }
        ActionID::ToggleAutoGo => {
            if app.autogo.any_active() {
                *app.autogo.follow.mode_mut() = AutoGoOpMode::Off;
                *app.autogo.timecode.mode_mut() = AutoGoOpMode::Off;
            } else {
                *app.autogo.follow.mode_mut() = AutoGoOpMode::Ctrl;
                *app.autogo.timecode.mode_mut() = AutoGoOpMode::Ctrl;
            }
        }
        ActionID::CommandLineBackspace => app.commandline.backspace(),
        ActionID::CommandLineCancel => app.commandline.clear(),
        ActionID::GoCue(cue) => app.go_cue(&cue),
        ActionID::ToggleAutoscroll => app.autoscroll = !app.autoscroll,
    }
}

fn try_cue_last(app: &mut TekstApp) -> Option<()> {
    let seq = app.selected_sequence_mut().as_mut()?;
    seq.sequence.cue_pointer = seq.sequence.cues.len() - 1;
    None
}

fn try_cue_first(app: &mut TekstApp) -> Option<()> {
    let seq = app.selected_sequence_mut().as_mut()?;
    seq.sequence.cue_pointer = 0;
    None
}

fn try_cue_add(app: &mut TekstApp, num: usize) -> Option<()> {
    if let Some(seq) = app.selected_sequence_mut() {
        seq.sequence.cue_pointer = seq
            .sequence
            .cue_pointer
            .saturating_add(num)
            .min(seq.sequence.cues.len() - 1);
    }
    None
}

fn try_cue_sub(app: &mut TekstApp, num: usize) -> Option<()> {
    let seq = app.selected_sequence_mut().as_mut()?;
    seq.sequence.cue_pointer = seq.sequence.cue_pointer.saturating_sub(num);
    None
}

fn try_cue_mark_next(app: &mut TekstApp) -> Option<()> {
    app.selected_sequence_mut()
        .as_mut()?
        .sequence
        .goto_next_mark();
    None
}

fn try_cue_mark_prev(app: &mut TekstApp) -> Option<()> {
    app.selected_sequence_mut()
        .as_mut()?
        .sequence
        .goto_prev_mark();
    None
}

pub fn all_default_shortcuts() -> ShortcutMap {
    let mut shortcuts = ShortcutMap::new();

    add_patch_shortcuts(&mut shortcuts);
    add_commandline_shortcuts(&mut shortcuts);
    shortcuts.add(ActionID::Go, press(Key::Space));
    shortcuts.add(ActionID::SelectCueUp(1), press(Key::ArrowUp));
    shortcuts.add(ActionID::SelectCueDown(1), press(Key::ArrowDown));
    shortcuts.add(ActionID::GoCue(Cue::default()), press(Key::Delete));

    shortcuts.add(ActionID::ToggleAutoscroll, press(Key::R));
    shortcuts.add(ActionID::ToggleAutoGo, press(Key::T));

    shortcuts
}

fn add_commandline_shortcuts(shortcuts: &mut ShortcutMap) -> Option<()> {
    for i in 0..10 {
        shortcuts.add(
            ActionID::CommandLineAppendChar(i.to_string().chars().next()?),
            press(Key::from_name(&i.to_string())?),
        );
    }
    shortcuts.add(ActionID::CommandLineAppendChar('.'), press(Key::Period));

    shortcuts.add(ActionID::CommandLinePlease, press(Key::Enter));
    shortcuts.add(ActionID::CommandLineBackspace, press(Key::Backspace));
    shortcuts.add(ActionID::CommandLineCancel, press(Key::Escape));

    shortcuts.add(
        ActionID::CommandLineAppendToken(CommandLineToken::Ident("<this>".to_string())),
        press(Key::Backtick),
    );
    shortcuts.add(
        ActionID::CommandLineAppendToken(CommandLineToken::Ident("<mark>".to_string())),
        press(Key::M),
    );

    for (token, key) in CMDLINE_HOTKEYS {
        shortcuts.add(ActionID::CommandLineAppendToken(token), press(key));
    }

    None
}

fn add_patch_shortcuts(shortcuts: &mut ShortcutMap) {
    for (i, key) in [
        Key::F1,
        Key::F2,
        Key::F3,
        Key::F4,
        Key::F5,
        Key::F6,
        Key::F7,
        Key::F8,
        Key::F9,
        Key::F10,
        Key::F11,
        Key::F12,
    ]
    .iter()
    .enumerate()
    {
        shortcuts.add(ActionID::ChangeSequence(i), press(*key));
    }
}

fn press(key: Key) -> Shortcut {
    Shortcut::new(Some(KeyboardShortcut::new(Modifiers::NONE, key)), None)
}
fn _ctrl(key: Key) -> Shortcut {
    Shortcut::new(Some(KeyboardShortcut::new(Modifiers::CTRL, key)), None)
}
fn _shift(key: Key) -> Shortcut {
    Shortcut::new(Some(KeyboardShortcut::new(Modifiers::SHIFT, key)), None)
}
