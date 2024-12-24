#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Quality {
    Diminished,
    Minor,
    Major,
    Augmented,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Extension {
    Sixth,
    MinorSeventh,
    MajorSeventh,
    Ninth,
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

#[derive(Debug, Clone, PartialEq)]
pub struct ModifierStack {
    qualities: Vec<Quality>,
    extensions: Vec<Extension>,
    inversions: Vec<Inversion>,
}

impl ModifierStack {
    pub fn new() -> Self {
        Self {
            qualities: vec![Quality::Major],
            extensions: vec![],
            inversions: vec![Inversion::Root],
        }
    }

    pub fn update(&mut self, modifier: Modifier, is_pressed: bool) {
        match modifier {
            Modifier::Quality(q) => self.update_quality(q, is_pressed),
            Modifier::Extension(e) => self.update_extension(e, is_pressed),
            Modifier::Inversion(i) => self.update_inversion(i, is_pressed),
        }
    }

    pub fn get_current_state(&self) -> Vec<Modifier> {
        let mut modifiers = Vec::new();

        if let Some(q) = self.qualities.last() {
            modifiers.push(Modifier::Quality(*q));
        }
        if let Some(e) = self.extensions.last() {
            modifiers.push(Modifier::Extension(*e));
        }
        if let Some(i) = self.inversions.last() {
            modifiers.push(Modifier::Inversion(*i));
        }

        if modifiers.is_empty() {
            return vec![Modifier::Quality(Quality::Major)];
        }

        modifiers
    }

    fn update_quality(&mut self, quality: Quality, is_pressed: bool) {
        if is_pressed {
            self.qualities.push(quality);
        } else {
            self.qualities.retain(|&m| m != quality);
        }
        if self.qualities.is_empty() {
            self.qualities.push(Quality::Major);
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
}

use std::fmt;

impl fmt::Display for ModifierStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let qualities = self.qualities.last().map_or("", |q| match q {
            Quality::Major => "maj",
            Quality::Minor => "min",
            Quality::Diminished => "dim",
            Quality::Augmented => "aug",
        });

        let extensions = self.extensions.last().map_or("", |e| match e {
            Extension::Sixth => "6",
            Extension::MinorSeventh => "7",
            Extension::MajorSeventh => "maj7",
            Extension::Ninth => "9",
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
