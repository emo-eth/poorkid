mod chord;
mod modifier; // You'll need this too since chord.rs uses it
use device_query::{DeviceQuery, DeviceState, Keycode};
use midir::MidiOutput;
use midir::os::unix::VirtualOutput;
use modifier::{Extension, Inversion, Modifier, ModifierStack, Quality};
use std::io;
use std::sync::{Arc, Mutex};
use std::{error::Error, io::Write, time::Duration};
use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tokio::sync::mpsc;
use tokio::time::sleep;
use wmidi::{MidiMessage, Note, U7, Velocity};

// Add this helper function before the run() function
async fn schedule_note(tx: mpsc::Sender<wmidi::MidiMessage<'_>>) {
    let _ = tx
        .send(MidiMessage::NoteOn(
            wmidi::Channel::Ch1,
            Note::C3,
            Velocity::from(U7::from_u8_lossy(127)),
        ))
        .await;
    sleep(Duration::from_millis(100)).await;
    let _ = tx
        .send(MidiMessage::NoteOff(
            wmidi::Channel::Ch1,
            Note::C3,
            Velocity::from(U7::from_u8_lossy(0)),
        ))
        .await;
}

// The #[tokio::main] attribute sets up Tokio's async runtime
// This runtime manages all concurrent tasks and handles their scheduling
// Think of it as an event loop that efficiently juggles multiple operations
#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => (),
        Err(err) => println!("Error: {}", err),
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    // Initialize MIDI output with name "Poorkid"
    let midi_out = MidiOutput::new("Poorkid")?;

    // Create a virtual MIDI port that other applications can connect to
    println!("\nCreating virtual port...");
    let port = midi_out.create_virtual("Poorkid")?;
    let mut modifier_stack = ModifierStack::new();

    let mut handle_modifier = move |modifier: Modifier, is_pressed: bool| {
        println!(
            "Modifier {:?} {}",
            modifier,
            if is_pressed { "pressed" } else { "released" }
        );
        // Add your modifier handling logic here
        modifier_stack.update(modifier, is_pressed);
    };

    // Create a channel for sending MIDI messages between tasks
    // mpsc = Multi-Producer, Single-Consumer channel
    // - tx (transmitter): Can be cloned to allow multiple senders
    // - rx (receiver): Only one receiver can exist
    // The channel has a buffer size of 32 messages
    let (tx, mut rx) = mpsc::channel::<wmidi::MidiMessage>(32);

    // Wrap the MIDI port in Arc and Mutex for thread-safe sharing
    // - Arc (Atomic Reference Counting): Allows sharing between threads
    // - Mutex: Ensures only one thread can access the port at a time
    let port = Arc::new(Mutex::new(port));

    // Initialize device state for keyboard monitoring
    let device_state = DeviceState::new();
    let mut previous_keys = Vec::new();

    // Spawn a separate async task to handle MIDI message processing
    // The 'move' keyword transfers ownership of rx and port_clone into the closure
    // This task runs concurrently with the main input loop
    let midi_task = tokio::spawn(async move {
        // Process messages until receiving Reset signal
        while let Some(msg) = rx.recv().await {
            // Check if it's a Reset message, which signals program termination
            if matches!(msg, MidiMessage::Reset) {
                break;
            }

            // Otherwise, send the MIDI message through the port
            if let Ok(mut locked_port) = port.lock() {
                let _ = locked_port.send(&msg.to_vec());
            }
        }
    });

    println!("\nPress numpad keys for modifiers, 'Q' to quit...");

    // Main input loop
    'main: loop {
        // Get current keys being pressed
        let keys = device_state.get_keys();

        // Check for newly pressed keys (key down events)
        for key in keys.iter() {
            if !previous_keys.contains(key) {
                match key {
                    Keycode::Q => {
                        tx.send(MidiMessage::Reset).await?;
                        break 'main;
                    }
                    // Numpad keys
                    Keycode::Numpad7 => {
                        handle_modifier(Modifier::Quality(Quality::Diminished), true)
                    }
                    Keycode::Numpad8 => handle_modifier(Modifier::Quality(Quality::Minor), true),
                    Keycode::Numpad9 => handle_modifier(Modifier::Quality(Quality::Major), true),
                    Keycode::NumpadSubtract => {
                        handle_modifier(Modifier::Quality(Quality::Augmented), true)
                    }
                    Keycode::Numpad4 => {
                        handle_modifier(Modifier::Extension(Extension::Sixth), true)
                    }
                    Keycode::Numpad5 => {
                        handle_modifier(Modifier::Extension(Extension::MinorSeventh), true)
                    }
                    Keycode::Numpad6 => {
                        handle_modifier(Modifier::Extension(Extension::MajorSeventh), true)
                    }
                    Keycode::NumpadAdd => {
                        handle_modifier(Modifier::Extension(Extension::Ninth), true)
                    }
                    Keycode::Numpad1 => handle_modifier(Modifier::Inversion(Inversion::Root), true),
                    Keycode::Numpad2 => {
                        handle_modifier(Modifier::Inversion(Inversion::First), true)
                    }
                    Keycode::Numpad3 => {
                        handle_modifier(Modifier::Inversion(Inversion::Second), true)
                    }
                    Keycode::NumpadEnter => {
                        handle_modifier(Modifier::Inversion(Inversion::Third), true)
                    }
                    _ => schedule_note(tx.clone()).await,
                }
            }
        }

        // Check for released keys (key up events)
        for key in previous_keys.iter() {
            if !keys.contains(key) {
                match key {
                    Keycode::Numpad7 => {
                        handle_modifier(Modifier::Quality(Quality::Diminished), false)
                    }
                    Keycode::Numpad8 => handle_modifier(Modifier::Quality(Quality::Minor), false),
                    Keycode::Numpad9 => handle_modifier(Modifier::Quality(Quality::Major), false),
                    Keycode::NumpadAdd => {
                        handle_modifier(Modifier::Quality(Quality::Augmented), false)
                    }
                    Keycode::Numpad4 => {
                        handle_modifier(Modifier::Extension(Extension::Sixth), false)
                    }
                    Keycode::Numpad5 => {
                        handle_modifier(Modifier::Extension(Extension::MinorSeventh), false)
                    }
                    Keycode::Numpad6 => {
                        handle_modifier(Modifier::Extension(Extension::MajorSeventh), false)
                    }
                    Keycode::NumpadSubtract => {
                        handle_modifier(Modifier::Extension(Extension::Ninth), false)
                    }
                    Keycode::Numpad1 => {
                        handle_modifier(Modifier::Inversion(Inversion::Root), false)
                    }
                    Keycode::Numpad2 => {
                        handle_modifier(Modifier::Inversion(Inversion::First), false)
                    }
                    Keycode::Numpad3 => {
                        handle_modifier(Modifier::Inversion(Inversion::Second), false)
                    }
                    Keycode::NumpadEnter => {
                        handle_modifier(Modifier::Inversion(Inversion::Third), false)
                    }
                    _ => {}
                }
            }
        }

        // Update previous keys for next iteration
        previous_keys = keys;

        // Small delay to prevent CPU spinning
        sleep(Duration::from_millis(1)).await;
    }

    // Wait for MIDI processing task to complete before exiting
    midi_task.await.map_err(|e| Box::new(e) as Box<dyn Error>)?;
    Ok(())
}
