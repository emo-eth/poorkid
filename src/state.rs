use crate::modifier::ModifierStack;
use crate::theory::{Key, Scale};
use wmidi::Note;

// Core state structs
#[derive(Debug, Clone)]
pub struct GlobalState {
    pub key: Key,
    pub bpm: f32,
    pub perform: Perform,
    pub perform_params: PerformState,
    pub modifier_state: ModifierStack,
    pub active_notes: Vec<Note>,
    pub page: Page,
}

impl GlobalState {
    fn new() -> Self {
        Self {
            key: Key::new(Note::C4, Scale::Ionian),
            bpm: 120.0,
            perform: Perform::None,
            perform_params: PerformState::new(),
            modifier_state: ModifierStack::new(),
            page: Page::One,
            active_notes: Vec::new(),
        }
    }
}

// Rotary control enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotaryControl {
    Root(Note),
    Scale(Scale),
    Bpm(u16),
    Perform(Perform),
    PerformParam(PerformParam),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerformState {
    pub spacing: u8,
    pub arpeggiator: ArpeggiatorState,
}

impl PerformState {
    fn new() -> Self {
        Self {
            spacing: 20, // default value
            arpeggiator: ArpeggiatorState::new(),
        }
    }

    fn update(&mut self, param: PerformParam) {
        match param {
            PerformParam::StrumSpacing(value) => self.spacing = value,
            PerformParam::ArpeggioDirection(dir) => self.arpeggiator.direction = dir,
            PerformParam::ArpeggioRate(rate) => self.arpeggiator.rate = rate,
            PerformParam::None => (),
        }
    }

    fn get_value(&self, param_type: PerformParam) -> PerformParam {
        match param_type {
            PerformParam::StrumSpacing(_) => PerformParam::StrumSpacing(self.spacing),
            PerformParam::ArpeggioDirection(_) => {
                PerformParam::ArpeggioDirection(self.arpeggiator.direction)
            }
            PerformParam::ArpeggioRate(_) => PerformParam::ArpeggioRate(self.arpeggiator.rate),
            PerformParam::None => PerformParam::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArpeggiatorState {
    pub direction: ArpeggioDirection,
    pub rate: Rate,
    pub index: usize,
}

impl ArpeggiatorState {
    fn new() -> Self {
        Self {
            direction: ArpeggioDirection::Up,
            rate: Rate::Eighth,
            index: 0,
        }
    }
}

// Enums
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Perform {
    None,
    Strum,
    Strum2Octave,
    Arpeggio,
    Arpeggio2Octave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformParam {
    None,
    StrumSpacing(u8),
    ArpeggioDirection(ArpeggioDirection),
    ArpeggioRate(Rate),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rate {
    Quarter = 4,
    Eighth = 8,
    Twelfth = 12,
    Sixteenth = 16,
    TwentyFourth = 24,
    ThirtySecond = 32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArpeggioDirection {
    Up,
    Down,
    UpDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    One,
    Two,
}

impl Page {
    fn get_controls(&self) -> [RotaryControl; 4] {
        match self {
            Page::One => [
                RotaryControl::Root(Note::C4),
                RotaryControl::Scale(Scale::Ionian),
                RotaryControl::Bpm(120),
                RotaryControl::Perform(Perform::None),
            ],
            Page::Two => [
                RotaryControl::PerformParam(PerformParam::StrumSpacing(20)),
                RotaryControl::PerformParam(PerformParam::ArpeggioDirection(ArpeggioDirection::Up)),
                RotaryControl::PerformParam(PerformParam::ArpeggioRate(Rate::Eighth)),
                RotaryControl::PerformParam(PerformParam::None), // You might want to change this last one
            ],
        }
    }
}
