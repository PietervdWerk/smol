use rdev::{EventType, Key, listen};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Enum to define the two types of shortcuts you wanted.
enum ShortcutType {
    // A set of keys that must be pressed at the same time.
    Combination(HashSet<Key>),
    // A single key that must be pressed twice in succession within a given timeout.
    Succession { key: Key, timeout: Duration },
}

// This struct holds the information for a registered shortcut.
struct Shortcut {
    shortcut_type: ShortcutType,
    // The code to run when the shortcut is triggered.
    callback: Box<dyn Fn() + Send + Sync>,
}

/// Manages keyboard shortcuts and listens for their activation.
pub struct KeyboardManager {
    shortcuts: Arc<Mutex<Vec<Shortcut>>>,
    // Keeps track of the keys currently being held down.
    pressed_keys: Arc<Mutex<HashSet<Key>>>,
    // Records the time of the last key press to check for succession shortcuts.
    last_key_press_time: Arc<Mutex<Option<Instant>>>,
    // Records the last key that was pressed to ensure succession is direct.
    last_key_pressed: Arc<Mutex<Option<Key>>>,
}

impl KeyboardManager {
    /// Creates a new, empty KeyboardManager.
    pub fn new() -> Self {
        Self {
            shortcuts: Arc::new(Mutex::new(Vec::new())),
            pressed_keys: Arc::new(Mutex::new(HashSet::new())),
            last_key_press_time: Arc::new(Mutex::new(None)),
            last_key_pressed: Arc::new(Mutex::new(None)),
        }
    }

    /// Registers a combination shortcut.
    pub fn register_combination<F>(&mut self, keys: Vec<Key>, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let shortcut = Shortcut {
            shortcut_type: ShortcutType::Combination(keys.into_iter().collect()),
            callback: Box::new(callback),
        };
        self.shortcuts.lock().unwrap().push(shortcut);
    }

    /// Registers a succession shortcut for a key pressed twice in a row.
    pub fn register_succession<F>(&mut self, key: Key, timeout: Duration, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let shortcut = Shortcut {
            shortcut_type: ShortcutType::Succession { key, timeout },
            callback: Box::new(callback),
        };
        self.shortcuts.lock().unwrap().push(shortcut);
    }

    /// Starts listening for keyboard events in a new thread.
    /// This function will not block the main thread.
    pub fn start_listening(&self) {
        let shortcuts = Arc::clone(&self.shortcuts);
        let pressed_keys = Arc::clone(&self.pressed_keys);
        let last_key_press_time = Arc::clone(&self.last_key_press_time);
        let last_key_pressed = Arc::clone(&self.last_key_pressed);

        thread::spawn(move || {
            listen(move |event| {
                let mut pressed_keys = pressed_keys.lock().unwrap();
                let mut last_key_press_time = last_key_press_time.lock().unwrap();
                let mut last_key_pressed = last_key_pressed.lock().unwrap();
                let shortcuts = shortcuts.lock().unwrap();

                match event.event_type {
                    EventType::KeyPress(key) => {
                        // println!("Pressed: {:?}", key);
                        pressed_keys.insert(key);

                        // Check for combination shortcuts.
                        for shortcut in shortcuts.iter() {
                            if let ShortcutType::Combination(keys) = &shortcut.shortcut_type {
                                if keys.is_subset(&pressed_keys) {
                                    (shortcut.callback)();
                                }
                            }
                        }

                        // Check for succession shortcuts.
                        if let Some(last_press) = *last_key_press_time {
                            for shortcut in shortcuts.iter() {
                                if let ShortcutType::Succession {
                                    key: succession_key,
                                    timeout,
                                } = &shortcut.shortcut_type
                                {
                                    // Check if the current key and the last key pressed are the same as the shortcut key,
                                    // and if the press is within the timeout.
                                    if *succession_key == key
                                        && Some(*succession_key) == *last_key_pressed
                                        && last_press.elapsed() <= *timeout
                                    {
                                        (shortcut.callback)();
                                    }
                                }
                            }
                        }

                        // Update the last key press time and key.
                        *last_key_press_time = Some(Instant::now());
                        *last_key_pressed = Some(key);
                    }
                    EventType::KeyRelease(key) => {
                        pressed_keys.remove(&key);
                    }
                    _ => {}
                }
            })
            .expect("Failed to listen for keyboard events");
        });
    }
}
