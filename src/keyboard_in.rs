use crate::modifier::{self, Extension, Inversion, Quality};
use device_query::{CallbackGuard, DeviceEvents, DeviceState, Keycode};
use modifier::{Modifier, ModifierStack};
use std::error::Error;
use std::sync::{Arc, RwLock};

pub struct KeyboardIn {
    key_up_handler: CallbackGuard<Box<dyn Fn(&Keycode) + Send + Sync>>,
    key_down_handler: CallbackGuard<Box<dyn Fn(&Keycode) + Send + Sync>>,
}

pub async fn run_input(
    modifier_stack: Arc<RwLock<ModifierStack>>,
) -> Result<KeyboardIn, Box<dyn Error>> {
    let handle_modifier = move |modifier: Modifier, is_pressed: bool| {
        println!(
            "Modifier {:?} {}",
            modifier,
            if is_pressed { "pressed" } else { "released" }
        );
        if let Ok(mut data) = modifier_stack.write() {
            data.update(modifier, is_pressed);
        }
    };
    let handle_key = move |key: Keycode, pressed: bool| {
        // Get current keys being pressed

        // Check for newly pressed keys (key down events)
        match key {
            // Numpad keys
            Keycode::Numpad7 => handle_modifier(Modifier::Quality(Quality::Diminished), true),
            Keycode::Numpad8 => handle_modifier(Modifier::Quality(Quality::Minor), true),
            Keycode::Numpad9 => handle_modifier(Modifier::Quality(Quality::Major), true),
            Keycode::NumpadSubtract => handle_modifier(Modifier::Quality(Quality::Augmented), true),
            Keycode::Numpad4 => handle_modifier(Modifier::Extension(Extension::Sixth), true),
            Keycode::Numpad5 => handle_modifier(Modifier::Extension(Extension::MinorSeventh), true),
            Keycode::Numpad6 => handle_modifier(Modifier::Extension(Extension::MajorSeventh), true),
            Keycode::NumpadAdd => handle_modifier(Modifier::Extension(Extension::Ninth), true),
            Keycode::Numpad1 => handle_modifier(Modifier::Inversion(Inversion::Root), true),
            Keycode::Numpad2 => handle_modifier(Modifier::Inversion(Inversion::First), true),
            Keycode::Numpad3 => handle_modifier(Modifier::Inversion(Inversion::Second), true),
            Keycode::NumpadEnter => handle_modifier(Modifier::Inversion(Inversion::Third), true),
            _ => {}
        }
    };
    let handle_key_clone = handle_key.clone();

    // Create a channel for sending MIDI messages between tasks
    // mpsc = Multi-Producer, Single-Consumer channel
    // - tx (transmitter): Can be cloned to allow multiple senders
    // - rx (receiver): Only one receiver can exist
    // The channel has a buffer size of 32 messages
    // let (tx, mut rx) = mpsc::channel::<(Keycode, bool)>(32);
    // let tx_clone = tx.clone();

    // Initialize device state for keyboard monitoring
    let device_state = DeviceState::new();
    println!("\nPress numpad keys for modifiers, 'Q' to quit...");
    let key_up_handler: CallbackGuard<Box<dyn Fn(&Keycode) + Send + Sync>> = device_state
        .on_key_up(Box::new(move |&key| {
            println!("Key up: {:?}", key);
            handle_key(key, false);
        }));
    let key_down_handler: CallbackGuard<Box<dyn Fn(&Keycode) + Send + Sync>> = device_state
        .on_key_down(Box::new(move |&key| {
            println!("Key down: {:?}", key);
            handle_key_clone(key, true);
        }));

    // let result = tokio::spawn(async move {
    //     println!("Keyboard input task started");
    //     let key_up_handler = key_up_handler;
    //     let key_down_handler = key_down_handler;
    //     while let Some((key, is_pressed)) = rx.recv().await {
    //         println!("Received key: {:?}; pressed: {}", key, is_pressed);
    //     }
    //     println!("Keyboard input task ended");
    // });

    Ok(KeyboardIn {
        key_up_handler,
        key_down_handler,
    })

    // Wait for MIDI processing task to complete before exiting
}
