use cached::proc_macro::cached;
use wmidi::{Note, U7};

#[derive(Debug, Clone)]
pub struct Key {
    root: Note,
    pub scale: Scale,
}

impl Key {
    pub fn new(root: Note, scale: Scale) -> Self {
        let note_index = u8::from(root) % 12;
        let base_root = Note::from_u8_lossy(note_index);
        Self {
            root: base_root,
            scale,
        }
    }

    pub fn root(&self) -> Note {
        self.root
    }

    pub fn get_notes(&self) -> Vec<Note> {
        self.scale.get_notes(self.root)
    }

    pub fn get_triad(&self, root: Note) -> Vec<Note> {
        // get index of root in scale, or next closest above note
        let index = self.get_notes().iter().position(|n| n >= &root).unwrap();
        // get notes offset 2 and 4 from index
        let mut notes = vec![root];

        let scale_notes = &self.get_notes();
        if let Some(third) = scale_notes.get(index + 2) {
            notes.push(*third);
        }
        if let Some(fifth) = scale_notes.get(index + 4) {
            notes.push(*fifth);
        }
        notes
    }

    pub fn get_interval(&self, root: Note, interval: u8) -> Option<Note> {
        let notes = self.get_notes();
        let index = notes.iter().position(|n| n >= &root).unwrap();
        if let Some(interval_note) = notes.get((index + interval as usize) % notes.len()) {
            Some(*interval_note)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum Scale {
    Ionian,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
    Locrian,
    HarmonicMinor,
    LocrianNatural6,
    IonianSharp5,
    DorianSharp4,
    PhrygianDominant,
    LydianSharp9,
    AlteredDiminished,
    HarmonicMajor,
    MelodicMinor,
}

impl Scale {
    pub const Major: Scale = Scale::Ionian;
    pub const Minor: Scale = Scale::Aeolian;

    pub fn get_intervals(&self) -> Vec<u8> {
        match self {
            Scale::Ionian => vec![2, 2, 1, 2, 2, 2, 1],
            Scale::Dorian => Scale::get_mode_intervals(&Scale::Ionian.get_intervals(), 1),
            Scale::Phrygian => Scale::get_mode_intervals(&Scale::Ionian.get_intervals(), 2),
            Scale::Lydian => Scale::get_mode_intervals(&Scale::Ionian.get_intervals(), 3),
            Scale::Mixolydian => Scale::get_mode_intervals(&Scale::Ionian.get_intervals(), 4),
            Scale::Aeolian => Scale::get_mode_intervals(&Scale::Ionian.get_intervals(), 5),
            Scale::Locrian => Scale::get_mode_intervals(&Scale::Ionian.get_intervals(), 6),
            Scale::HarmonicMinor => vec![2, 1, 2, 2, 1, 3, 1],
            Scale::LocrianNatural6 => {
                Scale::get_mode_intervals(&Scale::HarmonicMinor.get_intervals(), 1)
            }
            Scale::IonianSharp5 => Scale::get_mode_intervals(&Scale::Ionian.get_intervals(), 2),
            Scale::DorianSharp4 => Scale::get_mode_intervals(&Scale::Ionian.get_intervals(), 3),
            Scale::PhrygianDominant => Scale::get_mode_intervals(&Scale::Ionian.get_intervals(), 4),
            Scale::LydianSharp9 => Scale::get_mode_intervals(&Scale::Ionian.get_intervals(), 5),
            Scale::AlteredDiminished => {
                Scale::get_mode_intervals(&Scale::Ionian.get_intervals(), 6)
            }
            Scale::HarmonicMajor => vec![2, 2, 1, 2, 1, 3, 1],
            Scale::MelodicMinor => vec![2, 1, 2, 2, 2, 2, 1],
        }
    }

    fn get_mode_intervals(intervals: &Vec<u8>, offset: usize) -> Vec<u8> {
        let mut new_intervals = intervals.clone();
        new_intervals.rotate_left(offset % intervals.len());
        new_intervals
    }

    /// Get the tonic notes of the scale in octave -2, ordered by their note value
    pub fn get_tonic_notes(&self, root: Note) -> Vec<Note> {
        let note_value = u8::from(root);
        let base_note = note_value % 12;
        let intervals = self.get_intervals();
        let mut notes = Vec::new();
        let base_root = Note::from_u8_lossy(base_note);
        notes.push(base_root);
        for interval in intervals {
            // get the note at the interval, then wrap it around mod 12
            let transposed_note = notes.last().unwrap().step(interval as i8).unwrap();
            let normalized_note = Note::from_u8_lossy((u8::from(transposed_note) % 12));
            notes.push(normalized_note);
        }
        // sort the vector; c-2 will be first, B-2 will be last
        notes.sort();
        notes
    }

    /// Get ALL notes in the scale, from octave -2 to octave 8
    pub fn get_notes(&self, root: Note) -> Vec<Note> {
        let note_value = u8::from(root);
        let base_note = note_value % 12;
        let intervals = self.get_intervals();
        let mut notes = self.get_tonic_notes(root);
        // get index of base_note in notes
        let base_note_index = notes
            .iter()
            .position(|n| n == &Note::from_u8_lossy(base_note))
            .unwrap();
        // determine last interval index using base_note_index and length of notes
        // eg 7 - 1 - 0 = 6; 6 % 7 = 6, next step is to the octave
        // eg 7 - 1 - 1 = 5; 5 % 7 = 5, next step is to the 7th
        // eg 7 - 1 - 6 = 0; 0 % 7 = 0, next step is to the second
        let mut current_interval_idx = notes.len() - 1 - base_note_index;
        let mut current_note = *notes.last().unwrap();
        while u8::from(current_note) < 127 {
            let interval = intervals[current_interval_idx % intervals.len()];
            match current_note.step(interval as i8) {
                Ok(next_note) => {
                    current_note = next_note;
                    notes.push(current_note);
                    current_interval_idx += 1;
                }
                Err(_) => break,
            }
        }

        notes
    }
}
