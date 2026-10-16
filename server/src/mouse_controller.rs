use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};
use std::sync::mpsc::{channel, Sender};
use std::thread;

enum Command {
    Move(i32, i32),
    Click(Button),
}

// Enigo is !Send on macOS, so it lives on one thread and receives commands.
pub struct MouseController(Sender<Command>);

impl MouseController {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        thread::spawn(move || {
            let mut enigo =
                Enigo::new(&Settings::default()).expect("Failed to create Enigo instance");
            for cmd in rx {
                let result = match cmd {
                    Command::Move(dx, dy) => enigo.move_mouse(dx, dy, Coordinate::Rel),
                    Command::Click(button) => enigo.button(button, Direction::Click),
                };
                if let Err(e) = result {
                    eprintln!("Mouse error: {}", e);
                }
            }
        });
        Self(tx)
    }

    pub fn click(&self, button: Button) -> Result<(), String> {
        self.0.send(Command::Click(button)).map_err(|e| e.to_string())
    }

    pub fn mouse_move(&self, dx: i32, dy: i32) -> Result<(), String> {
        self.0.send(Command::Move(dx, dy)).map_err(|e| e.to_string())
    }
}
