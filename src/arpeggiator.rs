use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::state::{GlobalState, Perform};

fn create(state: Arc<RwLock<GlobalState>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        println!("Arpeggiator running");
        let state_readable = state.clone();
        let state_writable = state;

        loop {
            let state_read = state_readable.read().await;
            match state_read.perform {
                Perform::Arpeggio => {
                    let index = state_read.perform_params.arpeggiator.index;
                    if let Some(note) = state_read
                        .active_notes
                        .get(index % state_read.active_notes.len())
                    {
                        // play note

                        let seconds_per_minute = 60.0;
                        let ms_per_second = 1000.0;
                        let beats_per_bar = 4.0;

                        // block to automatically drop state_write
                        {
                            // Update index for next note
                            let mut state_write = state_writable.write().await;
                            // do not modulo index because notes may be added or removed via modifiers
                            state_write.perform_params.arpeggiator.index = index + 1;
                        }
                        // Wait for next note duration
                        let wait_ms = (beats_per_bar * ms_per_second * seconds_per_minute
                            / state_read.bpm)
                            / f32::from(state_read.perform_params.arpeggiator.rate as u8);
                        drop(state_read);
                        sleep(Duration::from_millis(wait_ms as u64)).await;
                    }
                }
                _ => {
                    drop(state_read);
                    sleep(Duration::from_millis(10)).await;
                }
            }
        }
    })
}
