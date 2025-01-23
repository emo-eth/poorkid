mod arpeggiator;
mod keyboard_in;
mod midi;
mod midi_in;
mod modifier;
mod modifier_handler;
mod state;
mod theory;
// use device_query::{DeviceQuery, DeviceState, Keycode};
use midir::MidiOutput;
use midir::os::unix::VirtualOutput;
use modifier::{Modifier, ModifierStack};
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::sync::{RwLock, mpsc};
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
    let midi_out_port = midi_out.create_virtual("Poorkid")?;

    let (midi_bytes_sender, mut midi_bytes_receiver) = mpsc::channel::<Vec<u8>>(32);

    // Wrap the MIDI port in Arc and Mutex for thread-safe sharing
    // - Arc (Atomic Reference Counting): Allows sharing between threads
    // - Mutex: Ensures only one thread can access the port at a time
    let midi_out_port_threadsafe = Arc::new(Mutex::new(midi_out_port));

    let modifier_stack = Arc::new(RwLock::new(ModifierStack::new()));

    let midi_intercept_task =
        midi_in::run_input(midi_bytes_sender.clone(), modifier_stack.clone()).await?;
    let modifier_handler_task = modifier_handler::handle_modifiers(modifier_stack.clone()).await?;
    // let _keyboard_in = keyboard_in::run_input(modifier_stack).await?;

    // Spawn a task to handle MIDI output
    let midi_output_task = tokio::spawn({
        let port = midi_out_port_threadsafe.clone();
        async move {
            while let Some(message) = midi_bytes_receiver.recv().await {
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

    let bytes = vec![1, 2, 3];
    let (send, mut receive) = mpsc::channel::<Arc<MidiMessage>>(32);
    let message = Arc::new(MidiMessage::from_bytes(&bytes).unwrap().to_owned());
    // create a receiver tokio thread
    tokio::spawn(async move {
        let port = midi_out_port_threadsafe.clone();
        async move {
            while let Some(message) = receive.recv().await {
                println!("Sending to output port: {:?}", message);
                if let Ok(mut port) = port.lock() {
                    if let Err(e) = port.send(message.bytes()) {
                        println!("Error sending MIDI message: {:?}", e);
                    }
                }
            }
        }
    });

    // Wait for tasks to complete
    tokio::try_join!(
        async {
            midi_intercept_task
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error>)
        },
        async {
            midi_output_task
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error>)
        },
        async {
            modifier_handler_task
                .await
                .map_err(|e| Box::new(e) as Box<dyn Error>)
        }
    )?;

    Ok(())
}
