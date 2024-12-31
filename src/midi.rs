use midir::{MidiInput, MidiInputPort, MidiOutput, MidiOutputPort};
use std::error::Error;

pub fn get_midi_in_port() -> Result<MidiInputPort, Box<dyn Error>> {
    // Get the first available input port
    let midi_in = MidiInput::new("Poorkid Input")?;
    let in_ports = midi_in.ports();
    let input_port = {
        let mut selected_port = None;
        for port in in_ports.iter() {
            if let Ok(name) = midi_in.port_name(port) {
                if name == "OP-XY" || name == "OP-XY Bluetooth" {
                    println!("Found OP-XY");
                    selected_port = Some(port);
                    break;
                }
            }
        }
        selected_port
            .or_else(|| in_ports.get(0))
            .ok_or_else(|| Box::<dyn Error>::from("no input port found"))?
            .clone()
    };
    Ok(input_port)
}
