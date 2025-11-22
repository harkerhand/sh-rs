use crate::{IS_WAITING_FOR_INPUT, print_error, prompt};
use std::sync::atomic::Ordering;
use tokio::signal::unix::{SignalKind, signal};

#[cfg(unix)]
pub async fn sigint_handler() {
    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to set up SIGINT handler");
    loop {
        sigint.recv().await;
        if IS_WAITING_FOR_INPUT.load(Ordering::SeqCst) {
            print_error!("\n\r");
            prompt::print_prompt();
        }
    }
}
