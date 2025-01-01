use crate::midi::get_midi_in_port;
use crate::modifier::ModifierStack;
use midir::{MidiInput, MidiOutput};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use wmidi::{Channel, MidiMessage, Note, U7};

pub struct ChordStatus {
    pub roots: HashMap<Channel, HashMap<Note, Vec<Note>>>,
}

impl ChordStatus {
    pub fn new() -> Self {
        Self {
            roots: HashMap::new(),
        }
    }

    pub fn insert(&mut self, channel: Channel, note: Note, chord: &Vec<Note>) {
        self.roots
            .entry(channel)
            .or_insert(HashMap::new())
            .insert(note, chord.clone());
    }
}

async fn transform_message(
    stack: Arc<RwLock<ModifierStack>>,
    status: Arc<RwLock<ChordStatus>>,
    tx: mpsc::Sender<Vec<u8>>,
    message: Vec<u8>,
) {
    println!("Transforming message: {:?}", message);
    let midi_message = match MidiMessage::from_bytes(&message) {
        Ok(msg) => msg,
        Err(e) => {
            println!("Error parsing MIDI message: {:?}", e);
            return;
        }
    };
    let mut off = false;
    let messages = match midi_message {
        MidiMessage::NoteOn(channel, note, velocity) => {
            println!("NoteOn: {:?}", midi_message);
            let notes = stack.read().unwrap().get_notes(note);
            if let Ok(mut status) = status.write() {
                status.insert(channel, note, &notes);
            } else {
                println!("Failed to acquire write lock on status");
                return;
            }
            let mut messages = vec![midi_message];
            messages.extend(
                notes
                    .iter()
                    .map(|note| MidiMessage::NoteOn(channel, *note, velocity)),
            );
            messages
        }
        MidiMessage::NoteOff(channel, note, velocity) => {
            println!("NoteOff: {:?}", midi_message);
            off = true;
            // get existing notes and remove from status
            let notes = {
                if let Ok(mut status) = status.write() {
                    match status.roots.get(&channel) {
                        Some(channel_notes) => match channel_notes.get(&note) {
                            Some(notes) => notes.clone(),
                            None => {
                                println!(
                                    "No notes found for note {:?} on channel {:?}",
                                    note, channel
                                );
                                return;
                            }
                        },
                        None => {
                            println!("No notes found for channel {:?}", channel);
                            return;
                        }
                    }
                } else {
                    println!("Failed to acquire write lock on status");
                    return;
                }
            };

            let mut messages = vec![midi_message];
            messages.extend(
                notes
                    .iter()
                    .map(|note| MidiMessage::NoteOff(channel, *note, velocity)),
            );
            messages
        }
        _ => {
            vec![]
        }
    };

    futures::future::join_all(messages.iter().enumerate().map(|(i, midi_message)| {
        schedule_midi_message(
            tx.clone(),
            Duration::from_millis(if off { 0 } else { i as u64 * 10 }),
            midi_message.to_vec(),
        )
    }))
    .await;
}

async fn schedule_midi_message(tx: mpsc::Sender<Vec<u8>>, delay: Duration, midi_message: Vec<u8>) {
    tokio::time::sleep(delay).await;
    if let Err(e) = tx.send(midi_message).await {
        println!("Failed to send MIDI message: {:?}", e);
    }
}

pub async fn run_input(
    tx: mpsc::Sender<Vec<u8>>,
    modifier_stack: Arc<RwLock<ModifierStack>>,
) -> Result<JoinHandle<()>, Box<dyn Error>> {
    // create new virtual output port
    let midi_out = MidiOutput::new("Poorkid")?;

    // Create a virtual MIDI port that other applications can connect to
    // println!("\nCreating virtual port...");
    // let mut out_port = midi_out.create_virtual("Poorkid")?;

    // Create channel for communication between MIDI callback and async task
    let (callback_tx, mut callback_rx) = mpsc::channel::<Vec<u8>>(1024); // Increased buffer size

    // Create a channel for error reporting
    let (error_tx, mut error_rx) = mpsc::channel::<String>(32);
    let error_tx_clone = error_tx.clone();

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
                let _ = error_tx.try_send(format!("Callback send error: {:?}", e));
            }
        },
        (),
    )?;

    let input_task = tokio::spawn(async move {
        println!("MIDI input task started");
        // move ownership of conn to the async task, otherwise it will be closed when returning
        let conn = _conn;
        let status = Arc::new(RwLock::new(ChordStatus::new()));

        // Spawn a monitoring task
        let monitor_task = tokio::spawn(async move {
            while let Some(error) = error_rx.recv().await {
                println!("Error received: {}", error);
            }
        });

        let result: Result<(), Box<dyn Error + Send + Sync>> = async {
            while let Some(message) = callback_rx.recv().await {
                println!("Received message, sending to main receiver");
                match MidiMessage::from_bytes(&message) {
                    Ok(midi_message) => {
                        println!("Sending MIDI message outer: {:?}", midi_message);
                        transform_message(
                            modifier_stack.clone(),
                            status.clone(),
                            tx.clone(),
                            midi_message.to_vec(),
                        )
                        .await;
                    }
                    Err(e) => {
                        let err_msg = format!(
                            "Failed to parse MIDI message: {:?} - Error: {:?}",
                            message, e
                        );
                        println!("{}", err_msg);
                        error_tx_clone.send(err_msg).await?;
                    }
                }
            }
            Ok(())
        }
        .await;

        if let Err(e) = result {
            println!("MIDI input task error: {:?}", e);
            error_tx_clone
                .send(format!("MIDI input task error: {:?}", e))
                .await
                .unwrap_or_default();
        }

        println!("MIDI input task ended");
        conn.close();
    });

    Ok(input_task)
}
