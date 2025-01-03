use std::sync::{Arc, RwLock};

use tokio::task::JoinHandle;

use crate::state::GlobalState;

fn create(state: Arc<RwLock<GlobalState>>) -> JoinHandle<()> {
    tokio::spawn(async move {
        let state = state.read().unwrap();
        println!("Arpeggiator running");
    })
}
