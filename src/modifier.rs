use std::fmt;

use device_query::Keycode;
use wmidi::{ControlFunction, MidiMessage, Note};

#[derive(Debug, Clone, Copy, PartialEq)]
struct Triad {
    third: i8,
    fifth: i8,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quality {
    Diminished,
    Minor,
    Major,
    Augmented,
    Sus2,
    Sus4,
}

impl Quality {
    fn get_triad(&self) -> Triad {
        match self {
            Quality::Diminished => Triad { third: 3, fifth: 6 },
            Quality::Minor => Triad { third: 3, fifth: 7 },
            Quality::Major => Triad { third: 4, fifth: 7 },
            Quality::Augmented => Triad { third: 4, fifth: 8 },
            Quality::Sus2 => Triad { third: 2, fifth: 7 },
            Quality::Sus4 => Triad { third: 5, fifth: 7 },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Extension {
    FlatSixth,
    Sixth,
    MinorSeventh,
    MajorSeventh,
    FlatNinth,
    Ninth,
    SharpNinth,
    FlatEleventh,
    Eleventh,
    SharpEleventh,
    FlatThirteenth,
    Thirteenth,
    SharpThirteenth,
}

impl Extension {
    fn get_semitones(&self) -> i8 {
        use Extension::*;
        match self {
            FlatSixth => 8,
            Sixth => 9,
            MinorSeventh => 10,
            MajorSeventh => 11,
            FlatNinth => 13,
            Ninth => 14,
            SharpNinth => 15,
            FlatEleventh => 16,
            Eleventh => 17,
            SharpEleventh => 18,
            FlatThirteenth => 20,
            Thirteenth => 21,
            SharpThirteenth => 22,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Inversion {
    Root,
    First,
    Second,
    Third,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modifier {
    Quality(Quality),
    Extension(Extension),
    Inversion(Inversion),
}

/// Allow for both keyboard and MIDI input to select modifiers
pub enum MappingInput<'a> {
    Keycode(Keycode),
    MidiMessage(MidiMessage<'a>),
}

pub trait ModifierMapping {
    fn get_modifier(input: MappingInput) -> Option<(Modifier, bool)>;
}

// Example implementations
pub struct KeyboardMapping;
pub struct OPXYMapping;

impl ModifierMapping for KeyboardMapping {
    fn get_modifier(input: MappingInput) -> Option<(Modifier, bool)> {
        match input {
            MappingInput::Keycode(key) => match key {
                Keycode::Numpad7 => Some((Modifier::Quality(Quality::Diminished), true)),
                Keycode::Numpad8 => Some((Modifier::Quality(Quality::Minor), true)),
                Keycode::Numpad9 => Some((Modifier::Quality(Quality::Major), true)),
                Keycode::NumpadSubtract => Some((Modifier::Quality(Quality::Augmented), true)),
                Keycode::Numpad4 => Some((Modifier::Extension(Extension::Sixth), true)),
                Keycode::Numpad5 => Some((Modifier::Extension(Extension::MinorSeventh), true)),
                Keycode::Numpad6 => Some((Modifier::Extension(Extension::MajorSeventh), true)),
                Keycode::NumpadAdd => Some((Modifier::Extension(Extension::Ninth), true)),
                Keycode::Numpad1 => Some((Modifier::Inversion(Inversion::Root), true)),
                Keycode::Numpad2 => Some((Modifier::Inversion(Inversion::First), true)),
                Keycode::Numpad3 => Some((Modifier::Inversion(Inversion::Second), true)),
                Keycode::NumpadEnter => Some((Modifier::Inversion(Inversion::Third), true)),
                _ => None,
            },
            _ => None,
        }
    }
}

impl ModifierMapping for OPXYMapping {
    fn get_modifier(input: MappingInput) -> Option<(Modifier, bool)> {
        match input {
            MappingInput::MidiMessage(msg) => match msg {
                MidiMessage::ControlChange(_, function, value) => match u8::from(function.0) {
                    7 => Some((Modifier::Quality(Quality::Major), u8::from(value) > 0)),
                    8 => Some((Modifier::Quality(Quality::Minor), u8::from(value) > 0)),
                    9 => Some((Modifier::Quality(Quality::Diminished), u8::from(value) > 0)),
                    10 => Some((Modifier::Quality(Quality::Augmented), u8::from(value) > 0)),
                    61 => Some((Modifier::Extension(Extension::Sixth), u8::from(value) > 0)),
                    62 => Some((
                        Modifier::Extension(Extension::MinorSeventh),
                        u8::from(value) > 0,
                    )),
                    63 => Some((
                        Modifier::Extension(Extension::MajorSeventh),
                        u8::from(value) > 0,
                    )),
                    64 => Some((Modifier::Extension(Extension::Ninth), u8::from(value) > 0)),
                    _ => None,
                },
                _ => None,
            }, // implement MIDI mapping logic here
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModifierStack {
    qualities: Vec<Quality>,
    extensions: Vec<Extension>,
    inversions: Vec<Inversion>,
}

impl ModifierStack {
    pub fn new() -> Self {
        Self {
            qualities: vec![],
            extensions: vec![],
            inversions: vec![],
        }
    }

    pub fn update(&mut self, modifier: Modifier, is_pressed: bool) {
        match modifier {
            Modifier::Quality(q) => self.update_quality(q, is_pressed),
            Modifier::Extension(e) => self.update_extension(e, is_pressed),
            Modifier::Inversion(i) => self.update_inversion(i, is_pressed),
        }
    }

    fn update_quality(&mut self, quality: Quality, is_pressed: bool) {
        if is_pressed {
            self.qualities.push(quality);
        } else {
            self.qualities.retain(|&m| m != quality);
        }
    }

    fn update_extension(&mut self, extension: Extension, is_pressed: bool) {
        if is_pressed {
            self.extensions.push(extension);
        } else {
            self.extensions.retain(|&m| m != extension);
        }
    }

    fn update_inversion(&mut self, inversion: Inversion, is_pressed: bool) {
        if is_pressed {
            self.inversions.push(inversion);
        } else {
            self.inversions.retain(|&m| m != inversion);
        }
    }

    pub fn get_notes(&self, root: Note) -> Vec<Note> {
        let mut notes = Vec::new();
        if let Some(triad) = self.qualities.last() {
            notes.push(root.step(triad.get_triad().third).unwrap());
            notes.push(root.step(triad.get_triad().fifth).unwrap());
        }
        for &extension in self.extensions.iter() {
            notes.push(root.step(extension.get_semitones()).unwrap());
        }
        notes
    }
}

impl fmt::Display for ModifierStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let qualities = self.qualities.last().map_or("", |q| match q {
            Quality::Major => "maj",
            Quality::Minor => "min",
            Quality::Diminished => "dim",
            Quality::Augmented => "aug",
            Quality::Sus2 => "sus2",
            Quality::Sus4 => "sus4",
        });

        let extensions = self.extensions.last().map_or("", |e| match e {
            Extension::Sixth => "6",
            Extension::MinorSeventh => "7",
            Extension::MajorSeventh => "maj7",
            Extension::Ninth => "9",
            Extension::FlatSixth => "b6",
            Extension::FlatNinth => "b9",
            Extension::FlatEleventh => "b11",
            Extension::Eleventh => "11",
            Extension::SharpNinth => "#9",
            Extension::SharpEleventh => "#11",
            Extension::FlatThirteenth => "b13",
            Extension::Thirteenth => "13",
            Extension::SharpThirteenth => "#13",
        });

        let inversion = self.inversions.last().map_or("", |i| match i {
            Inversion::Root => "",
            Inversion::First => "/1",
            Inversion::Second => "/2",
            Inversion::Third => "/3",
        });

        write!(f, "{}{}{}", qualities, extensions, inversion)
    }
}
