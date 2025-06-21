mod keyboard;

use keyboard::KeyboardManager;
use rdev::Key;
use std::thread;
use std::time::Duration;

fn main() {
    // Create an instance of the KeyboardManager.
    let mut manager = KeyboardManager::new();

    // Register a succession shortcut for pressing Left Shift twice within 300ms.
    manager.register_succession(Key::ShiftLeft, Duration::from_millis(300), || {
        println!("Double Shift triggered!");
    });

    // Register a succession shortcut for pressing Control twice within 300ms.
    manager.register_succession(Key::Unknown(62), Duration::from_millis(300), || {
        println!("Double Control triggered!");
    });

    manager.register_combination(vec![Key::MetaLeft, Key::KeyS], || {
        println!("'CMD+S' combination triggered!");
    });

    println!("Listening for a double Shift press (Left Shift twice within 300ms)...");

    // Start listening for events in the background.
    manager.start_listening();

    // Keep the main thread alive to allow the listener to run.
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
