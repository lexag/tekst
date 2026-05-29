use crate::cue::Cue;
use std::{io::BufReader, path::PathBuf};

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct Sequence {
    pub name: String,
    pub cue_pointer: usize,
    pub cues: Vec<Cue>,
}

impl Sequence {
    pub fn example() -> Self {
        Self {
            name: "example".to_string(),
            cue_pointer: 0,
            cues: vec![Cue::default(), Cue::default(), Cue::default()],
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct SequenceSlot {
    pub path: PathBuf,
    pub sequence: Sequence,
}

impl SequenceSlot {
    pub fn load_from_path(path: PathBuf) -> Option<Self> {
        let mut slot = Self {
            path: path.clone(),
            sequence: Sequence {
                name: path
                    .clone()
                    .file_stem()
                    .unwrap_or_default()
                    .to_os_string()
                    .into_string()
                    .ok()?,
                cue_pointer: 0,
                cues: vec![],
            },
        };

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)
            .ok()?;
        for result in rdr.deserialize() {
            if let Ok(cue) = result {
                slot.sequence.cues.push(cue);
            } else {
                panic!("Failed loading: {}", result.unwrap_err())
            }
        }
        Some(slot)
    }

    pub fn save_to_path(&self, path: PathBuf) -> Option<()> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_path(path)
            .ok()?;
        for cue in &self.sequence.cues {
            wtr.serialize(cue).unwrap();
        }
        wtr.flush().ok()?;
        Some(())
    }
}
