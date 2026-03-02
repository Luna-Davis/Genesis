use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

use notify::{Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher};

pub struct DebouncedWatcher {
    _watcher: RecommendedWatcher,
}

pub struct DebouncedEvents {
    #[allow(dead_code)]
    pub events: Vec<Event>,
    pub settled_at: Instant,
}

const DEBOUNCE_MS: u64 = 5000;

impl DebouncedWatcher {
    /// Start watching `path` recursively. Emits debounced batches on the provided channel.
    pub fn watch(
        path: PathBuf,
        sender: crossbeam_channel::Sender<DebouncedEvents>,
    ) -> NotifyResult<Self> {
        let (tx, rx) = channel();

        let mut watcher = notify::recommended_watcher(move |res| {
            let _ = tx.send(res);
        })?;

        watcher.watch(&path, RecursiveMode::Recursive)?;

        thread::spawn(move || {
            let mut buffer: Vec<Event> = Vec::new();
            let mut last_event = Instant::now();

            loop {
                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(Ok(event)) => {
                        buffer.push(event);
                        last_event = Instant::now();
                    }
                    Ok(Err(_)) => continue,
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                        if !buffer.is_empty()
                            && last_event.elapsed() >= Duration::from_millis(DEBOUNCE_MS)
                        {
                            let events = std::mem::take(&mut buffer);
                            let _ = sender.send(DebouncedEvents {
                                events,
                                settled_at: Instant::now(),
                            });
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        });

        Ok(Self { _watcher: watcher })
    }
}
