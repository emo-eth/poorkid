use crate::modifier::ModifierStack;
use crate::theory::{Key, Scale};
use wmidi::Note;

#[derive(Debug, Clone)]
struct GlobalState {
    pub key: Key,
    pub bpm: u16,
    pub perform: Perform,
    pub perform_params: PerformParams,
    pub modifier_state: ModifierStack,
    pub page: Page,
}

impl GlobalState {
    fn new() -> Self {
        Self {
            key: Key::new(Note::C4, Scale::Ionian),
            bpm: 120,
            perform: Perform::None,
            perform_params: PerformParams::new(),
            modifier_state: ModifierStack::new(),
            page: Page::One,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Perform {
    None,
    Strum,
    Strum2Octave,
    Arpeggio,
    Arpeggio2Octave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PerformParam {
    Spacing(u8),
    Direction(ArpeggioDirection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PerformParams {
    pub spacing: u8,
    pub direction: ArpeggioDirection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArpeggioDirection {
    Up,
    Down,
    UpDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Page {
    One,
    Two,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageOne {
    Root,
    Scale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PageTwo {
    Perform,
    PerformParam(u8),
    Bpm,
}

impl PerformParams {
    fn new() -> Self {
        Self {
            spacing: 20,                      // default value
            direction: ArpeggioDirection::Up, // default value
        }
    }

    fn update(&mut self, param: PerformParam) {
        match param {
            PerformParam::Spacing(value) => self.spacing = value,
            PerformParam::Direction(dir) => self.direction = dir,
        }
    }

    fn get_value(&self, param_type: PerformParam) -> PerformParam {
        match param_type {
            PerformParam::Spacing(_) => PerformParam::Spacing(self.spacing),
            PerformParam::Direction(_) => PerformParam::Direction(self.direction),
        }
    }
}
