use std::{thread, time::Duration};

use anyhow::Context;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

pub struct TextInjector;

impl TextInjector {
    pub fn new() -> Self {
        Self
    }

    pub fn inject_text(&self, text: &str) -> anyhow::Result<()> {
        let mut clipboard = arboard::Clipboard::new().context("clipboard init failed")?;
        clipboard
            .set_text(text.to_string())
            .context("clipboard set failed")?;
        thread::sleep(Duration::from_millis(85));

        let mut enigo = Enigo::new(&Settings::default()).context("enigo init failed")?;
        enigo.key(Key::Control, Direction::Press)?;
        enigo.key(Key::Unicode('v'), Direction::Click)?;
        enigo.key(Key::Control, Direction::Release)?;
        Ok(())
    }
}
