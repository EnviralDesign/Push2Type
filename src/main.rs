#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod config;
mod hotkey;
mod inject;
mod server;
mod stt;
mod tts;

use std::sync::{Arc, Mutex};

use app::{AppEvent, Push2TypeApp};
use audio::AudioRecorder;
use config::AppConfig;
use crossbeam_channel::unbounded;

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let config = AppConfig::load_or_create()?;
    let shared_config = Arc::new(Mutex::new(config));

    let (ui_event_tx, ui_event_rx) = unbounded::<AppEvent>();
    let (stt_tx, stt_rx) = unbounded::<Vec<i16>>();
    let (tts_tx, tts_rx) = unbounded::<tts::TtsRequest>();

    let recorder = Arc::new(AudioRecorder::new(ui_event_tx.clone())?);

    stt::spawn_stt_worker(
        shared_config.clone(),
        ui_event_tx.clone(),
        stt_rx,
        Arc::new(inject::TextInjector::new()),
        recorder.sample_rate(),
    );
    tts::spawn_tts_worker(shared_config.clone(), ui_event_tx.clone(), tts_rx);
    hotkey::spawn_hotkey_worker(
        shared_config.clone(),
        ui_event_tx.clone(),
        recorder.clone(),
        stt_tx.clone(),
    );
    let (initial_tts_bridge_enabled, initial_server_port) = {
        let cfg = shared_config.lock().expect("config lock");
        (cfg.tts_bridge_enabled, cfg.server_port)
    };
    let server_control = server::spawn_server_controller(
        initial_tts_bridge_enabled,
        initial_server_port,
        ui_event_tx.clone(),
        tts_tx.clone(),
    );

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([480.0, 280.0])
        .with_min_inner_size([420.0, 240.0])
        .with_always_on_top();
    if let Some(icon) = load_window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Push2Type Satellite (Rust)",
        native_options,
        Box::new(move |_cc| {
            Ok(Box::new(Push2TypeApp::new(
                shared_config,
                ui_event_rx,
                tts_tx,
                stt_tx,
                recorder,
                server_control,
            )))
        }),
    )
    .map_err(|e| anyhow::anyhow!("eframe run failed: {e}"))?;

    Ok(())
}

fn load_window_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../assets/app.png");
    let image = image::load_from_memory(bytes).ok()?.to_rgba8();
    let (width, height) = image.dimensions();
    Some(egui::IconData {
        rgba: image.into_raw(),
        width,
        height,
    })
}
