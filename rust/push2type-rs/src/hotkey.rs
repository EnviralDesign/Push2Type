use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
    thread,
};

use crossbeam_channel::Sender;
use rdev::{EventType, Key, listen};

use crate::{app::AppEvent, audio::AudioRecorder, config::AppConfig};

#[derive(Debug, Clone)]
struct HotkeySpec {
    require_ctrl: bool,
    require_shift: bool,
    require_alt: bool,
    require_meta: bool,
    key: Option<Key>,
}

#[derive(Default)]
struct KeyState {
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
    pressed_non_mod: HashSet<Key>,
}

pub fn spawn_hotkey_worker(
    config: Arc<Mutex<AppConfig>>,
    events: Sender<AppEvent>,
    recorder: Arc<AudioRecorder>,
    stt_tx: Sender<Vec<i16>>,
) {
    thread::spawn(move || {
        let hotkey_str = config
            .lock()
            .ok()
            .map(|c| c.hotkey.clone())
            .unwrap_or_else(|| "ctrl+shift".to_string());

        let spec = parse_hotkey_spec(&hotkey_str).unwrap_or_else(|| {
            let _ = events.send(AppEvent::Warning(format!(
                "hotkey '{}' invalid, defaulting to ctrl+shift",
                hotkey_str
            )));
            HotkeySpec {
                require_ctrl: true,
                require_shift: true,
                require_alt: false,
                require_meta: false,
                key: None,
            }
        });
        let _ = events.send(AppEvent::Info(format!("hotkey active: {}", hotkey_str)));

        let state = Arc::new(Mutex::new(KeyState::default()));
        let active = Arc::new(Mutex::new(false));
        let cb_events = events.clone();
        let cb_recorder = recorder.clone();
        let cb_state = state.clone();
        let cb_active = active.clone();
        let cb_stt_tx = stt_tx.clone();

        let result = listen(move |event| {
            let mut st = match cb_state.lock() {
                Ok(s) => s,
                Err(_) => return,
            };
            update_key_state(&mut st, &event.event_type);
            let now_active = is_hotkey_active(&st, &spec);
            let mut was_active = match cb_active.lock() {
                Ok(v) => v,
                Err(_) => return,
            };

            if !*was_active && now_active {
                cb_recorder.start_capture();
                let _ = cb_events.send(AppEvent::Listening(true));
                *was_active = true;
            } else if *was_active && !now_active {
                let audio = cb_recorder.stop_capture();
                let _ = cb_events.send(AppEvent::Listening(false));
                if !audio.is_empty() {
                    let _ = cb_stt_tx.send(audio);
                }
                *was_active = false;
            }
        });

        if let Err(e) = result {
            let _ = events.send(AppEvent::Error(format!("hotkey listener failed: {e:?}")));
        }
    });
}

fn parse_hotkey_spec(input: &str) -> Option<HotkeySpec> {
    let mut spec = HotkeySpec {
        require_ctrl: false,
        require_shift: false,
        require_alt: false,
        require_meta: false,
        key: None,
    };

    for token in input.split('+').map(|s| s.trim().to_lowercase()) {
        match token.as_str() {
            "ctrl" | "control" => spec.require_ctrl = true,
            "shift" => spec.require_shift = true,
            "alt" => spec.require_alt = true,
            "win" | "window" | "meta" | "super" => spec.require_meta = true,
            "space" => spec.key = Some(Key::Space),
            "enter" => spec.key = Some(Key::Return),
            _ if token.len() == 1 => {
                if let Some(ch) = token.chars().next() {
                    spec.key = map_alpha_numeric(ch.to_ascii_uppercase());
                }
            }
            _ => {}
        }
    }

    if !(spec.require_ctrl || spec.require_shift || spec.require_alt || spec.require_meta)
        && spec.key.is_none()
    {
        return None;
    }
    Some(spec)
}

fn map_alpha_numeric(c: char) -> Option<Key> {
    match c {
        'A' => Some(Key::KeyA),
        'B' => Some(Key::KeyB),
        'C' => Some(Key::KeyC),
        'D' => Some(Key::KeyD),
        'E' => Some(Key::KeyE),
        'F' => Some(Key::KeyF),
        'G' => Some(Key::KeyG),
        'H' => Some(Key::KeyH),
        'I' => Some(Key::KeyI),
        'J' => Some(Key::KeyJ),
        'K' => Some(Key::KeyK),
        'L' => Some(Key::KeyL),
        'M' => Some(Key::KeyM),
        'N' => Some(Key::KeyN),
        'O' => Some(Key::KeyO),
        'P' => Some(Key::KeyP),
        'Q' => Some(Key::KeyQ),
        'R' => Some(Key::KeyR),
        'S' => Some(Key::KeyS),
        'T' => Some(Key::KeyT),
        'U' => Some(Key::KeyU),
        'V' => Some(Key::KeyV),
        'W' => Some(Key::KeyW),
        'X' => Some(Key::KeyX),
        'Y' => Some(Key::KeyY),
        'Z' => Some(Key::KeyZ),
        '0' => Some(Key::Num0),
        '1' => Some(Key::Num1),
        '2' => Some(Key::Num2),
        '3' => Some(Key::Num3),
        '4' => Some(Key::Num4),
        '5' => Some(Key::Num5),
        '6' => Some(Key::Num6),
        '7' => Some(Key::Num7),
        '8' => Some(Key::Num8),
        '9' => Some(Key::Num9),
        _ => None,
    }
}

fn update_key_state(state: &mut KeyState, event: &EventType) {
    match event {
        EventType::KeyPress(key) => {
            set_modifier_state(state, *key, true);
            if !is_modifier_key(*key) {
                state.pressed_non_mod.insert(*key);
            }
        }
        EventType::KeyRelease(key) => {
            set_modifier_state(state, *key, false);
            if !is_modifier_key(*key) {
                state.pressed_non_mod.remove(key);
            }
        }
        _ => {}
    }
}

fn set_modifier_state(state: &mut KeyState, key: Key, pressed: bool) {
    match key {
        Key::ControlLeft | Key::ControlRight => state.ctrl = pressed,
        Key::ShiftLeft | Key::ShiftRight => state.shift = pressed,
        Key::Alt | Key::AltGr => state.alt = pressed,
        Key::MetaLeft | Key::MetaRight => state.meta = pressed,
        _ => {}
    }
}

fn is_modifier_key(key: Key) -> bool {
    matches!(
        key,
        Key::ControlLeft
            | Key::ControlRight
            | Key::ShiftLeft
            | Key::ShiftRight
            | Key::Alt
            | Key::AltGr
            | Key::MetaLeft
            | Key::MetaRight
    )
}

fn is_hotkey_active(state: &KeyState, spec: &HotkeySpec) -> bool {
    if spec.require_ctrl && !state.ctrl {
        return false;
    }
    if spec.require_shift && !state.shift {
        return false;
    }
    if spec.require_alt && !state.alt {
        return false;
    }
    if spec.require_meta && !state.meta {
        return false;
    }
    if let Some(key) = spec.key {
        return state.pressed_non_mod.contains(&key);
    }
    true
}
