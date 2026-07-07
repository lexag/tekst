use crate::cue::Cue;
use serde::ser::Error;
use std::path::PathBuf;

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
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

    pub fn find_ident(&self, ident: &String) -> Option<usize> {
        self.cues.iter().position(|c| c.ident.starts_with(ident))
    }

    pub fn find_next_mark(&self, start_idx: usize) -> Option<usize> {
        Some(
            self.cues[start_idx..]
                .iter()
                .position(|c| c.mark.is_some())?
                + start_idx,
        )
    }

    pub fn insert_cue(&mut self, idx: usize) {
        let new_cue = self.cues.insert_mut(
            idx,
            Cue {
                ident: "-".to_string(),
                ..Default::default()
            },
        );
    }

    //pub fn increment_ident(ident: &str) -> Option<String> {
    //    let mut parts: Vec<usize> = ident.split('.').filter_map(|s| s.parse().ok()).collect();
    //
    //    *parts.last_mut()? += 1;
    //
    //    let out: Vec<String> = parts.iter().map(ToString::to_string).collect();
    //
    //    Some(out.join("."))
    //}
    //
    //pub fn extend_ident(ident: &str) -> String {
    //    let mut s = ident.to_string();
    //    s.push_str(".1");
    //    s
    //}

    pub fn find_prev_mark(&self, end_idx: usize) -> Option<usize> {
        self.cues[..end_idx].iter().position(|c| c.mark.is_some())
    }

    pub fn goto_next_mark(&mut self) {
        if let Some(idx) = self.find_next_mark(self.cue_pointer) {
            self.cue_pointer = idx;
        } else {
            self.cue_pointer = self.cues.len() - 1;
        }
    }

    pub fn goto_prev_mark(&mut self) {
        if let Some(idx) = self.find_prev_mark(self.cue_pointer) {
            self.cue_pointer = idx;
        } else {
            self.cue_pointer = 0;
        }
    }

    pub fn goto_ident(&mut self, ident: &String) {
        if let Some(idx) = self.find_ident(ident) {
            self.cue_pointer = idx;
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, Debug)]
pub struct SequenceSlot {
    pub path: PathBuf,
    #[serde(skip)]
    pub sequence: Sequence,
}

impl SequenceSlot {
    pub fn load_from_path(path: PathBuf) -> Result<Self, csv::Error> {
        let mut slot = Self {
            path: path.clone(),
            sequence: Sequence {
                name: path
                    .clone()
                    .file_stem()
                    .unwrap_or_default()
                    .to_os_string()
                    .into_string()
                    .map_err(|e| csv::Error::custom("invalid path"))?,
                cue_pointer: 0,
                cues: vec![],
            },
        };

        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_path(path)?;
        for result in rdr.deserialize() {
            if let Ok(cue) = result {
                slot.sequence.cues.push(cue);
            } else {
                return Err(result.unwrap_err());
            }
        }
        Ok(slot)
    }

    pub fn save_to_path(&self, path: PathBuf) -> Result<(), csv::Error> {
        let mut wtr = csv::WriterBuilder::new()
            .has_headers(false)
            .from_path(path)
            .map_err(|e| csv::Error::custom("invalid path"))?;
        for cue in &self.sequence.cues {
            wtr.serialize(cue);
        }
        wtr.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn increment_ident() {
        assert_eq!(Sequence::increment_ident("0"), Some("1".to_string()));
        assert_eq!(Sequence::increment_ident("12"), Some("13".to_string()));
        assert_eq!(Sequence::increment_ident("8.1"), Some("8.2".to_string()));
        assert_eq!(
            Sequence::increment_ident("123.456.78"),
            Some("123.456.79".to_string())
        );
    }
}
