// Copyright © 2026 CTCycle
// Licensed under the MIT License.

use std::sync::atomic::{AtomicBool, Ordering};

static INTERRUPT_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn request_interrupt() {
    INTERRUPT_REQUESTED.store(true, Ordering::SeqCst);
}

pub fn take_interrupt_requested() -> bool {
    INTERRUPT_REQUESTED.swap(false, Ordering::SeqCst)
}

#[cfg(windows)]
pub struct ConptyInterruptMonitor {
    stop: std::sync::Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

#[cfg(windows)]
impl ConptyInterruptMonitor {
    fn start() -> Self {
        let stop = std::sync::Arc::new(AtomicBool::new(false));
        let thread_stop = std::sync::Arc::clone(&stop);
        let handle = std::thread::spawn(move || {
            use crossterm::event::{self, Event, KeyCode, KeyModifiers};
            use std::time::Duration;

            while !thread_stop.load(Ordering::SeqCst) {
                let has_event = event::poll(Duration::from_millis(100)).unwrap_or(false);
                if !has_event {
                    continue;
                }
                let Ok(event) = event::read() else {
                    break;
                };
                if matches!(
                    event,
                    Event::Key(key)
                        if key.code == KeyCode::Char('c')
                            && key.modifiers.contains(KeyModifiers::CONTROL)
                ) {
                    request_interrupt();
                    break;
                }
            }
        });
        Self {
            stop,
            handle: Some(handle),
        }
    }
}

#[cfg(windows)]
impl Drop for ConptyInterruptMonitor {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(windows)]
pub fn start_conpty_monitor() -> Option<ConptyInterruptMonitor> {
    (std::env::var_os("LLMETER_CONPTY").as_deref() == Some(std::ffi::OsStr::new("1")))
        .then(ConptyInterruptMonitor::start)
}

#[cfg(not(windows))]
pub fn start_conpty_monitor() -> Option<()> {
    None
}
