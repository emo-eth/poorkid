mod keyboard_in;
mod midi_in;
mod modifier;
// use device_query::{DeviceQuery, DeviceState, Keycode};
use midir::MidiOutput;
use midir::os::unix::VirtualOutput;
use modifier::{Modifier, ModifierStack};
use std::error::Error;
use std::sync::{Arc, Mutex, RwLock};
use tokio::sync::mpsc;
use wmidi::MidiMessage;

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
    // midi_in::run().await?;
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
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(32);

    // Wrap the MIDI port in Arc and Mutex for thread-safe sharing
    // - Arc (Atomic Reference Counting): Allows sharing between threads
    // - Mutex: Ensures only one thread can access the port at a time
    let port = Arc::new(Mutex::new(port));

    let modifier_stack = Arc::new(RwLock::new(ModifierStack::new()));

    println!("\nPress numpad keys for modifiers, 'Q' to quit...");
    let midi_in = midi_in::run_input(tx.clone(), modifier_stack.clone()).await?;
    let _keyboard_in = keyboard_in::run_input(modifier_stack).await?;

    // Spawn a task to handle MIDI output
    let output_task = tokio::spawn({
        let port = port.clone();
        async move {
            while let Some(message) = rx.recv().await {
                if let Ok(midi_message) = MidiMessage::from_bytes(&message) {
                    println!("Sending to output port: {:?}", midi_message);
                    if let Ok(mut port) = port.lock() {
                        if let Err(e) = port.send(&message) {
                            println!("Error sending MIDI message: {:?}", e);
                        }
                    }
                }
            }
        }
    });

    // Wait for tasks to complete
    tokio::try_join!(
        async { midi_in.await.map_err(|e| Box::new(e) as Box<dyn Error>) },
        async { output_task.await.map_err(|e| Box::new(e) as Box<dyn Error>) }
    )?;

    Ok(())
}
