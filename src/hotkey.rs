use std::{
    sync::{Arc, Mutex},
    thread,
};
#[cfg(not(target_os = "windows"))]
use std::collections::HashSet;

use crossbeam_channel::Sender;
#[cfg(not(target_os = "windows"))]
use rdev::{EventType, listen};
use rdev::Key;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, VK_LCONTROL, VK_LMENU, VK_LSHIFT, VK_LWIN, VK_RETURN, VK_RCONTROL,
    VK_RMENU, VK_RSHIFT, VK_RWIN, VK_SPACE,
};

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
#[cfg(not(target_os = "windows"))]
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

        #[cfg(target_os = "windows")]
        {
            run_windows_hotkey_loop(spec, events, recorder, stt_tx);
        }

        #[cfg(not(target_os = "windows"))]
        {
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
        }
    });
}

#[cfg(target_os = "windows")]
fn run_windows_hotkey_loop(
    spec: HotkeySpec,
    events: Sender<AppEvent>,
    recorder: Arc<AudioRecorder>,
    stt_tx: Sender<Vec<i16>>,
) {
    let _ = events.send(AppEvent::Info(
        "hotkey backend: windows key-state polling".to_string(),
    ));
    let mut was_active = false;

    loop {
        let now_active = is_hotkey_active_windows(&spec);
        if !was_active && now_active {
            recorder.start_capture();
            let _ = events.send(AppEvent::Listening(true));
            was_active = true;
        } else if was_active && !now_active {
            let audio = recorder.stop_capture();
            let _ = events.send(AppEvent::Listening(false));
            if !audio.is_empty() {
                let _ = stt_tx.send(audio);
            }
            was_active = false;
        }
        thread::sleep(std::time::Duration::from_millis(12));
    }
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
            "backtick" | "grave" => spec.key = Some(Key::BackQuote),
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
        '`' => Some(Key::BackQuote),
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

#[cfg(not(target_os = "windows"))]
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

#[cfg(not(target_os = "windows"))]
fn set_modifier_state(state: &mut KeyState, key: Key, pressed: bool) {
    match key {
        Key::ControlLeft | Key::ControlRight => state.ctrl = pressed,
        Key::ShiftLeft | Key::ShiftRight => state.shift = pressed,
        Key::Alt | Key::AltGr => state.alt = pressed,
        Key::MetaLeft | Key::MetaRight => state.meta = pressed,
        _ => {}
    }
}

#[cfg(not(target_os = "windows"))]
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

#[cfg(not(target_os = "windows"))]
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

#[cfg(target_os = "windows")]
fn is_hotkey_active_windows(spec: &HotkeySpec) -> bool {
    if spec.require_ctrl && !(is_vk_down(VK_LCONTROL as i32) || is_vk_down(VK_RCONTROL as i32)) {
        return false;
    }
    if spec.require_shift && !(is_vk_down(VK_LSHIFT as i32) || is_vk_down(VK_RSHIFT as i32)) {
        return false;
    }
    if spec.require_alt && !(is_vk_down(VK_LMENU as i32) || is_vk_down(VK_RMENU as i32)) {
        return false;
    }
    if spec.require_meta && !(is_vk_down(VK_LWIN as i32) || is_vk_down(VK_RWIN as i32)) {
        return false;
    }
    if let Some(key) = spec.key {
        if let Some(vk) = key_to_vk(key) {
            return is_vk_down(vk);
        }
        return false;
    }
    true
}

#[cfg(target_os = "windows")]
fn is_vk_down(vk: i32) -> bool {
    // High-order bit indicates key-down state.
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

#[cfg(target_os = "windows")]
fn key_to_vk(key: Key) -> Option<i32> {
    match key {
        Key::Space => Some(VK_SPACE as i32),
        Key::Return => Some(VK_RETURN as i32),
        Key::KeyA => Some('A' as i32),
        Key::KeyB => Some('B' as i32),
        Key::KeyC => Some('C' as i32),
        Key::KeyD => Some('D' as i32),
        Key::KeyE => Some('E' as i32),
        Key::KeyF => Some('F' as i32),
        Key::KeyG => Some('G' as i32),
        Key::KeyH => Some('H' as i32),
        Key::KeyI => Some('I' as i32),
        Key::KeyJ => Some('J' as i32),
        Key::KeyK => Some('K' as i32),
        Key::KeyL => Some('L' as i32),
        Key::KeyM => Some('M' as i32),
        Key::KeyN => Some('N' as i32),
        Key::KeyO => Some('O' as i32),
        Key::KeyP => Some('P' as i32),
        Key::KeyQ => Some('Q' as i32),
        Key::KeyR => Some('R' as i32),
        Key::KeyS => Some('S' as i32),
        Key::KeyT => Some('T' as i32),
        Key::KeyU => Some('U' as i32),
        Key::KeyV => Some('V' as i32),
        Key::KeyW => Some('W' as i32),
        Key::KeyX => Some('X' as i32),
        Key::KeyY => Some('Y' as i32),
        Key::KeyZ => Some('Z' as i32),
        Key::Num0 => Some('0' as i32),
        Key::Num1 => Some('1' as i32),
        Key::Num2 => Some('2' as i32),
        Key::Num3 => Some('3' as i32),
        Key::Num4 => Some('4' as i32),
        Key::Num5 => Some('5' as i32),
        Key::Num6 => Some('6' as i32),
        Key::Num7 => Some('7' as i32),
        Key::Num8 => Some('8' as i32),
        Key::Num9 => Some('9' as i32),
        _ => None,
    }
}
