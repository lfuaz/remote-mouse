use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};
use std::sync::Mutex;

pub struct MouseController(Mutex<Enigo>);

impl MouseController {
    pub fn new() -> Self {
        Self(Mutex::new(
            Enigo::new(&Settings::default()).expect("Failed to create Enigo instance"),
        ))
    }

    pub fn click(&self, button: Button) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|e| e.to_string())?
            .button(button, Direction::Click)
            .map_err(|e| e.to_string())
    }

    pub fn mouse_move(&self, dx: i32, dy: i32) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|e| e.to_string())?
            .move_mouse(dx, dy, Coordinate::Rel)
            .map_err(|e| e.to_string())
    }
}
