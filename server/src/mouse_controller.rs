use enigo::{Button, Coordinate, Direction, Enigo, Mouse, Settings};
use std::sync::{Arc, Mutex};

pub struct MouseController {
    enigo: Arc<Mutex<Enigo>>,
}

impl MouseController {
    pub fn new() -> Self {
        let settings = Settings::default();
        let enigo = Enigo::new(&settings).expect("Failed to create Enigo instance");
        Self {
            enigo: Arc::new(Mutex::new(enigo)),
        }
    }

    pub fn click(&self, button: Button) -> Result<(), String> {
        self.enigo
            .lock()
            .map_err(|e| e.to_string())?
            .button(button, Direction::Click)
            .map_err(|e| e.to_string())
    }

    pub fn mouse_move_batch(&self, moves: Vec<(i32, i32)>) -> Result<(), String> {
        let mut enigo = self.enigo.lock().map_err(|e| e.to_string())?;
        for (dx, dy) in moves {
            enigo
                .move_mouse(dx, dy, Coordinate::Rel)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}
