use crate::midi::get_midi_in_port;
use crate::modifier::{MappingInput, ModifierMapping, ModifierStack, OPXYMapping};
use midir::MidiInput;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use wmidi::MidiMessage;

pub async fn handle_modifiers(
    stack: Arc<RwLock<ModifierStack>>,
) -> Result<JoinHandle<()>, Box<dyn Error>> {
    let (callback_tx, mut callback_rx) = mpsc::channel::<Vec<u8>>(1024); // Increased buffer size

    let midi_in = MidiInput::new("Poorkid Input")?;
    let input_port = get_midi_in_port()?;

    // Create connection and move ownership of tx to the callback
    let _conn = midi_in.connect(
        &input_port,
        "midi-input",
        move |_stamp, message, _| {
            println!("Callback received message. Sending to receiver.");
            if let Err(e) = callback_tx.try_send(message.to_vec()) {
                println!("Failed to send message from callback: {:?}", e);
            }
        },
        (),
    )?;

    let input_task = tokio::spawn(async move {
        println!("MIDI input task started");
        // move ownership of conn to the async task, otherwise it will be closed when returning
        let conn = _conn;

        let result: Result<(), Box<dyn Error + Send + Sync>> = async {
            while let Some(message) = callback_rx.recv().await {
                println!("Received message, sending to main receiver");
                match MidiMessage::from_bytes(&message) {
                    Ok(midi_message) => {
                        println!("Sending MIDI message outer: {:?}", midi_message);
                        if let Some((modifier, pressed)) =
                            OPXYMapping::get_modifier(MappingInput::MidiMessage(midi_message))
                        {
                            println!("Received modifier: {:?}", modifier);
                            let mut data = stack.write().await;
                            data.update(modifier, pressed);
                        }
                    }
                    Err(e) => {
                        let err_msg = format!(
                            "Failed to parse MIDI message: {:?} - Error: {:?}",
                            message, e
                        );
                        println!("{}", err_msg);
                    }
                }
            }
            Ok(())
        }
        .await;

        if let Err(e) = result {
            println!("MIDI input task error: {:?}", e);
        }

        conn.close();
    });

    Ok(input_task)
}
