use crate::{
    app::{PatchPointer, TekstApp},
    cue::Cue,
    sequence::SequenceSlot,
};
use std::{fmt::Display, slice::Iter, thread::current};
use tekst_common::primitive::{Color, TextAlign, Transition};

#[derive(Clone)]
pub struct CommandLine {
    tokens: Vec<CommandLineToken>,
}

impl Default for CommandLine {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandLine {
    pub fn new() -> Self {
        Self {
            tokens: vec![CommandLineToken::CommandLineIndicator],
        }
    }

    pub fn push_token(&mut self, token: CommandLineToken) -> Option<bool> {
        let mut try_tokens = self.tokens.clone();
        try_tokens.push(token);
        Self::try_autoreplace(&mut try_tokens);
        if Self::is_valid_command_in_progress(&try_tokens) {
            self.tokens = try_tokens;
            return Some(true);
        }
        Some(false)
    }

    pub fn push_char(&mut self, c: char) -> Option<bool> {
        match self.tokens.last_mut()? {
            CommandLineToken::Ident(s) => s.push(c),
            CommandLineToken::ValueVal(v) => {
                *v = v
                    .saturating_mul(10)
                    .saturating_add(u8::try_from(c.to_digit(10)?).ok()?);
            }
            CommandLineToken::TransitionVal(v) => {
                *v = Transition::from(u8::try_from(c.to_digit(10)?).ok()?);
            }
            CommandLineToken::ColorVal(v) => {
                *v = match c {
                    '1' => Color::Red,
                    '2' => Color::Green,
                    '3' => Color::Amber,
                    _ => return Some(false),
                };
            }
            CommandLineToken::AlignVal(v) => {
                *v = match c {
                    '1' => TextAlign::Left,
                    '2' => TextAlign::Center,
                    '3' => TextAlign::Right,
                    _ => return Some(false),
                };
            }
            CommandLineToken::Color => {
                self.push_token(CommandLineToken::ColorVal(match c {
                    '1' => Color::Red,
                    '2' => Color::Green,
                    '3' => Color::Amber,
                    _ => return Some(false),
                }));
            }
            CommandLineToken::Align => {
                self.push_token(CommandLineToken::AlignVal(match c {
                    '1' => TextAlign::Left,
                    '2' => TextAlign::Center,
                    '3' => TextAlign::Right,
                    _ => return Some(false),
                }));
            }
            CommandLineToken::Transition => {
                self.push_token(CommandLineToken::TransitionVal(Transition::from(
                    u8::try_from(c.to_digit(10)?).ok()?,
                )));
            }
            CommandLineToken::Brightness => {
                self.push_token(CommandLineToken::ValueVal(
                    u8::try_from(c.to_digit(10)?).ok()?,
                ));
            }
            _ => {
                self.push_token(CommandLineToken::Ident(c.to_string()));
            }
        }
        Some(true)
    }

    pub fn clear(&mut self) {
        *self = Self::new()
    }

    pub fn backspace(&mut self) {
        if self.tokens.len() > 1 {
            self.tokens.pop();
        }
    }

    pub fn execute(&self, app: &mut TekstApp) -> Option<bool> {
        let mut it = self.tokens.iter();
        if *it.next()? != CommandLineToken::CommandLineIndicator {
            return Some(false);
        }
        match it.next()? {
            CommandLineToken::Goto => return self.execute_goto(app, it),
            CommandLineToken::Edit => return self.execute_edit(app, it),
            CommandLineToken::Delete => return self.execute_delete(app, it),
            CommandLineToken::Insert => return self.execute_insert(app, it),
            CommandLineToken::Append => return self.execute_append(app, it),
            _ => {}
        }
        Some(true)
    }

    fn execute_goto(&self, app: &mut TekstApp, mut it: Iter<CommandLineToken>) -> Option<bool> {
        let ident_type = it.next()?;
        match ident_type {
            CommandLineToken::Cue => {
                let idx = parse_single_ident(&mut it, app.selected_sequence()?)?;
                if let Some(seq) = app.selected_sequence_mut() {
                    seq.sequence.cue_pointer = idx;
                    app.autogo.dry_go_happened();
                }
            }
            CommandLineToken::Seq => {
                let ident_parsed = try_parse_seq_ident(app, it)?;
                if !(1..=12).contains(&ident_parsed) {
                    return None;
                }
                app.selected_sequence_idx = ident_parsed - 1;
                app.autogo.dry_go_happened();
            }
            _ => return Some(false),
        }
        Some(true)
    }

    fn execute_edit(&self, app: &mut TekstApp, mut it: Iter<CommandLineToken>) -> Option<bool> {
        let subject_type = it.next()?;
        match subject_type {
            CommandLineToken::Parent => execute_edit_parent(app, &mut it),
            CommandLineToken::Cue => {
                for cue_idx in parse_cue_ident(app, &mut it)? {
                    let cue = get_cue_by_index(app, cue_idx)?;

                    let personal_it = it.clone();
                    execute_edit_cue(cue, personal_it)?;
                }
                Some(true)
            }
            _ => None,
        }
    }

    fn execute_delete(&self, app: &mut TekstApp, mut it: Iter<CommandLineToken>) -> Option<bool> {
        match it.next()? {
            CommandLineToken::Seq => {
                let slot_number = try_parse_seq_ident(app, it)?;
                app.sequences[slot_number.saturating_sub(1)].take();
                Some(true)
            }
            CommandLineToken::Cue => {
                let rm_idxs = parse_cue_ident(app, &mut it)?;
                let sel_seq = &app.selected_sequence()?.sequence;
                if sel_seq.cues.len() > rm_idxs.len() {
                    let cues: Vec<Cue> = sel_seq
                        .cues
                        .iter()
                        .enumerate()
                        .filter_map(|(i, c)| {
                            if rm_idxs.contains(&i) {
                                None
                            } else {
                                Some(c.clone())
                            }
                        })
                        .collect();
                    app.selected_sequence_mut().as_mut()?.sequence.cues = cues;
                }
                Some(true)
            }
            _ => None,
        }
    }

    fn execute_insert(&self, app: &mut TekstApp, mut it: Iter<CommandLineToken>) -> Option<bool> {
        match it.next()? {
            CommandLineToken::Seq => {
                let slot_number = try_parse_seq_ident(app, it)?;
                Some(app.load_sequence_file(slot_number.saturating_sub(1)))
            }
            CommandLineToken::Cue => {
                let idx = parse_single_ident(&mut it, app.selected_sequence()?)?;
                app.selected_sequence_mut()
                    .as_mut()?
                    .sequence
                    .insert_cue(idx, false);

                Some(true)
            }
            _ => None,
        }
    }
    fn execute_append(&self, app: &mut TekstApp, mut it: Iter<CommandLineToken>) -> Option<bool> {
        if *it.next()? == CommandLineToken::Cue {
            let idx = parse_single_ident(&mut it, app.selected_sequence()?)?;
            app.selected_sequence_mut()
                .as_mut()?
                .sequence
                .insert_cue(idx + 1, true);
            return Some(true);
        }
        None
    }

    fn try_autoreplace(tokens: &mut Vec<CommandLineToken>) {
        for i in (0..tokens.len()).rev() {
            if let Some(new_tokens) = Self::autoreplace_lookup(&tokens[i..]) {
                tokens.resize(tokens.len() - i, CommandLineToken::CommandLineIndicator);
                if i >= tokens.len() {
                    continue;
                }
                *tokens = tokens[..i].to_vec();
                tokens.extend(new_tokens);
            }
        }
    }

    fn autoreplace_lookup(tokens: &[CommandLineToken]) -> Option<Vec<CommandLineToken>> {
        match tokens {
            [CommandLineToken::Edit, CommandLineToken::Ident(s)] => Some(vec![
                CommandLineToken::Edit,
                CommandLineToken::Cue,
                CommandLineToken::Ident(s.clone()),
            ]),
            [CommandLineToken::Append, CommandLineToken::Ident(s)] => Some(vec![
                CommandLineToken::Append,
                CommandLineToken::Cue,
                CommandLineToken::Ident(s.clone()),
            ]),
            [CommandLineToken::Insert, CommandLineToken::Ident(s)] => Some(vec![
                CommandLineToken::Insert,
                CommandLineToken::Cue,
                CommandLineToken::Ident(s.clone()),
            ]),
            [CommandLineToken::Goto, CommandLineToken::Ident(s)] => Some(vec![
                CommandLineToken::Goto,
                CommandLineToken::Cue,
                CommandLineToken::Ident(s.clone()),
            ]),
            _ => None,
        }
    }

    fn is_valid_command_in_progress(tokens: &[CommandLineToken]) -> bool {
        for i in 0..tokens.len() - 1 {
            if !tokens[i].valid_follower(&tokens[i + 1]) {
                return false;
            }
        }
        true
    }
}

fn get_cue_by_index(app: &mut TekstApp, cue_idx: usize) -> Option<&mut Cue> {
    app.selected_sequence_mut()
        .as_mut()?
        .sequence
        .cues
        .get_mut(cue_idx)
}

fn execute_edit_cue(cue: &mut Cue, mut it: Iter<'_, CommandLineToken>) -> Option<bool> {
    while let Some(prop) = it.next() {
        let val_token = it.next()?;
        match prop {
            CommandLineToken::Brightness => {
                cue.brightness = if let CommandLineToken::ValueVal(c) = val_token {
                    Some(*c)
                } else if *val_token == CommandLineToken::Parent {
                    None
                } else {
                    return None;
                }
            }
            CommandLineToken::Color => {
                cue.text_color = if let CommandLineToken::ColorVal(c) = val_token {
                    Some(*c)
                } else if *val_token == CommandLineToken::Parent {
                    None
                } else {
                    return None;
                }
            }
            CommandLineToken::Align => {
                cue.text_align = if let CommandLineToken::AlignVal(c) = val_token {
                    Some(*c)
                } else if *val_token == CommandLineToken::Parent {
                    None
                } else {
                    return None;
                }
            }
            CommandLineToken::Transition => {
                cue.fade_speed = if let CommandLineToken::TransitionVal(c) = val_token {
                    Some(*c)
                } else if *val_token == CommandLineToken::Parent {
                    None
                } else {
                    return None;
                }
            }
            CommandLineToken::Time => {
                if *val_token == CommandLineToken::Parent {
                    cue.autogo_delay_ms = None;
                    cue.autogo_timecode = None;
                }
            }
            _ => return None,
        }
    }
    Some(true)
}

fn execute_edit_parent(app: &mut TekstApp, it: &mut Iter<'_, CommandLineToken>) -> Option<bool> {
    let mut success = false;
    while let Some(prop) = it.next() {
        success = true;
        match prop {
            CommandLineToken::Brightness => {
                app.global_style.brightness = if let CommandLineToken::ValueVal(c) = it.next()? {
                    *c
                } else {
                    return None;
                }
            }
            CommandLineToken::Color => {
                app.global_style.text_color = if let CommandLineToken::ColorVal(c) = it.next()? {
                    *c
                } else {
                    return None;
                }
            }
            CommandLineToken::Align => {
                app.global_style.text_align = if let CommandLineToken::AlignVal(c) = it.next()? {
                    *c
                } else {
                    return None;
                }
            }
            CommandLineToken::Transition => {
                app.global_style.fade_speed = if let CommandLineToken::ValueVal(c) = it.next()?
                    && *c < 10
                {
                    Transition::from(*c)
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
    Some(success)
}

fn try_parse_seq_ident(app: &mut TekstApp, mut it: Iter<'_, CommandLineToken>) -> Option<usize> {
    if let Some(CommandLineToken::Ident(s)) = it.next()
        && let Ok(v) = s.parse::<usize>()
    {
        Some(v)
    } else {
        app.sequences.iter().position(Option::is_none)
    }
}

fn parse_cue_ident(app: &mut TekstApp, it: &mut Iter<'_, CommandLineToken>) -> Option<Vec<usize>> {
    println!("{:?}", it.clone());
    let current_cue = app.selected_sequence()?;
    let start_idx = parse_single_ident(it, current_cue)?;

    let mut personal_it = it.clone();
    if *personal_it.next()? == CommandLineToken::To {
        it.next();

        let end_idx = parse_single_ident(it, current_cue)?;
        return Some((start_idx..=end_idx).collect::<Vec<usize>>());
    }
    Some(vec![start_idx])
}

fn parse_single_ident(it: &mut Iter<'_, CommandLineToken>, cue: &SequenceSlot) -> Option<usize> {
    let CommandLineToken::Ident(ident) = it.next()? else {
        return None;
    };
    Some(match ident.as_str() {
        "<this>" => cue.sequence.cue_pointer,
        "<mark>" => cue.sequence.find_next_mark(cue.sequence.cue_pointer)?,
        _ => cue.sequence.find_ident(ident)?,
    })
}

impl Display for CommandLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for token in &self.tokens {
            s.push_str(&token.to_string());
            s.push(' ');
        }
        write!(f, "{s}")
    }
}

#[derive(
    Hash, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Clone, Debug,
)]
pub enum CommandLineToken {
    CommandLineIndicator,
    Goto,
    Delete,
    Seq,
    Cue,
    Insert,
    Append,
    Split,
    Merge,
    To,
    Edit,
    Parent,
    Align,
    Color,
    Transition,
    Time,
    Brightness,
    Ident(String),
    ColorVal(Color),
    ValueVal(u8),
    TransitionVal(Transition),
    AlignVal(TextAlign),
}

impl Display for CommandLineToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandLineIndicator => write!(f, ">"),
            Self::Goto => write!(f, "GOTO"),
            Self::Delete => write!(f, "DELETE"),
            Self::Seq => write!(f, "SEQ"),
            Self::Cue => write!(f, "CUE"),
            Self::To => write!(f, "TO"),
            Self::Insert => write!(f, "INSERT"),
            Self::Append => write!(f, "APPEND"),
            Self::Split => write!(f, "SPLIT"),
            Self::Merge => write!(f, "MERGE"),
            Self::Edit => write!(f, "EDIT"),
            Self::Parent => write!(f, "PARENT"),
            Self::Align => write!(f, "ALIGN"),
            Self::Color => write!(f, "COLOR"),
            Self::Time => write!(f, "TIME"),
            Self::Transition => write!(f, "TRANSIT"),
            Self::Brightness => write!(f, "BRIGHT"),
            Self::Ident(s) => write!(f, "{}", s),
            Self::ColorVal(v) => write!(f, "{}", v),
            Self::ValueVal(v) => write!(f, "{}", v),
            Self::TransitionVal(v) => write!(f, "{}", v),
            Self::AlignVal(v) => write!(f, "{}", v),
        }
    }
}

impl CommandLineToken {
    fn valid_follower(&self, f: &Self) -> bool {
        if f == self {
            return false;
        }
        match self {
            Self::CommandLineIndicator => {
                matches!(
                    f,
                    Self::Goto | Self::Delete | Self::Edit | Self::Insert | Self::Append
                )
            }
            Self::Goto => matches!(f, Self::Cue | Self::Seq),
            Self::Delete => matches!(f, Self::Cue | Self::Seq),
            Self::Seq => matches!(f, Self::Ident(..)),
            Self::Cue => matches!(f, Self::Ident(..)),
            Self::To => matches!(f, Self::Ident(..)),
            Self::Insert => matches!(f, Self::Cue | Self::Seq),
            Self::Append => matches!(f, Self::Cue),
            Self::Split => matches!(f, Self::Cue),
            Self::Merge => matches!(f, Self::Cue),
            Self::Edit => matches!(f, Self::Cue | Self::Parent),
            Self::Parent => matches!(
                f,
                Self::Align | Self::Color | Self::Brightness | Self::Transition
            ),
            Self::Align => matches!(f, Self::AlignVal(..) | Self::Parent),
            Self::Color => matches!(f, Self::ColorVal(..) | Self::Parent),
            Self::Time => matches!(f, Self::Parent),
            Self::Transition => matches!(f, Self::TransitionVal(..) | Self::Parent),
            Self::Brightness => matches!(f, Self::ValueVal(..) | Self::Parent),
            Self::Ident(_) => !matches!(
                f,
                Self::Ident(..)
                    | Self::Goto
                    | Self::Delete
                    | Self::Append
                    | Self::Insert
                    | Self::Edit
            ),
            Self::ColorVal(..)
            | Self::ValueVal(..)
            | Self::AlignVal(..)
            | Self::TransitionVal(..) => {
                matches!(
                    f,
                    Self::Align | Self::Transition | Self::Brightness | Self::Color
                )
            }
        }
    }
}
