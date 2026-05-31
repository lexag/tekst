use crate::{
    app::TekstApp,
    esds::{Color, TextAlign},
};
use std::fmt::Display;

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
                *v = v.saturating_mul(10).saturating_add(c.to_digit(10)? as u8)
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
            CommandLineToken::Fade | CommandLineToken::Brightness => {
                self.push_token(CommandLineToken::ValueVal(c.to_digit(10)? as u8));
            }
            _ => {
                self.push_token(CommandLineToken::Ident(c.to_string()));
            }
        };
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

    pub fn execute(&self, app: &mut TekstApp) {}

    fn try_autoreplace(tokens: &mut Vec<CommandLineToken>) {
        for i in (0..tokens.len()).rev() {
            if let Some(new_tokens) = Self::autoreplace_lookup(&tokens[i..]) {
                tokens.resize(tokens.len() - i, CommandLineToken::CommandLineIndicator);
                *tokens = tokens[..i].to_vec();
                tokens.extend(new_tokens);
            }
        }
    }

    fn autoreplace_lookup(tokens: &[CommandLineToken]) -> Option<Vec<CommandLineToken>> {
        match tokens {
            [CommandLineToken::Goto, CommandLineToken::Ident(s)] => Some(vec![
                CommandLineToken::Goto,
                CommandLineToken::Cue,
                CommandLineToken::Ident(s.to_string()),
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

impl Display for CommandLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        for token in &self.tokens {
            s.push_str(&token.to_string());
            s.push(' ');
        }
        write!(f, "{}", s)
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
    Art,
    Cue,
    PatchCue,
    Insert,
    Edit,
    Parent,
    Align,
    Color,
    Fade,
    Brightness,
    Ident(String),
    ColorVal(Color),
    ValueVal(u8),
    AlignVal(TextAlign),
}

impl Display for CommandLineToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandLineIndicator => write!(f, ">"),
            Self::Goto => write!(f, "GOTO"),
            Self::Delete => write!(f, "DELETE"),
            Self::Seq => write!(f, "SEQ"),
            Self::Art => write!(f, "ART"),
            Self::Cue => write!(f, "CUE"),
            Self::PatchCue => write!(f, "PATCHCUE"),
            Self::Insert => write!(f, "INSERT"),
            Self::Edit => write!(f, "EDIT"),
            Self::Parent => write!(f, "PARENT"),
            Self::Align => write!(f, "ALIGN"),
            Self::Color => write!(f, "COLOR"),
            Self::Fade => write!(f, "FADE"),
            Self::Brightness => write!(f, "BRIGHT"),
            Self::Ident(s) => write!(f, "{}", s),
            Self::ColorVal(v) => write!(f, "{}", v),
            Self::ValueVal(v) => write!(f, "{}", v),
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
                matches!(f, Self::Goto | Self::Delete | Self::Edit | Self::Insert)
            }
            Self::Goto => matches!(f, Self::Cue | Self::Seq | Self::Art | Self::PatchCue,),
            Self::Delete => matches!(f, Self::Cue | Self::Seq | Self::Art | Self::PatchCue,),
            Self::Seq => matches!(f, Self::Ident(..)),
            Self::Art => matches!(f, Self::Ident(..)),
            Self::Cue => matches!(f, Self::Ident(..)),
            Self::PatchCue => {
                matches!(f, Self::Ident(..))
            }
            Self::Insert => matches!(f, Self::Cue | Self::Seq | Self::Art | Self::PatchCue,),
            Self::Edit => matches!(
                f,
                Self::Cue | Self::Seq | Self::Art | Self::PatchCue | Self::Parent
            ),
            Self::Parent => matches!(f, Self::Align | Self::Color | Self::Brightness | Self::Fade),
            Self::Align => matches!(f, Self::AlignVal(..) | Self::Parent),
            Self::Color => matches!(f, Self::ColorVal(..) | Self::Parent),
            Self::Fade => matches!(f, Self::ValueVal(..) | Self::Parent),
            Self::Brightness => matches!(f, Self::ValueVal(..) | Self::Parent),
            Self::Ident(_) => !matches!(f, Self::Ident(..),),
            Self::ColorVal(..) | Self::ValueVal(..) | Self::AlignVal(..) => {
                matches!(f, Self::Align | Self::Fade | Self::Brightness | Self::Color)
            }
        }
    }
}
