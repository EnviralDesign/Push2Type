use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender};
use eframe::egui;

use crate::{
    audio::AudioRecorder,
    config::{AppConfig, Provider},
    server::ServerControl,
    tts::{SpeakRequest, TtsRequest},
};

#[derive(Debug, Clone)]
pub enum AppEvent {
    Info(String),
    Warning(String),
    Error(String),
    Listening(bool),
    SttBusy(bool),
    TtsBusy(bool),
    LastTranscript(String),
    LastSpoken(String),
    ServerOnline(String),
    ServerOffline,
}

pub struct Push2TypeApp {
    config: Arc<Mutex<AppConfig>>,
    events: Receiver<AppEvent>,
    tts_tx: Sender<TtsRequest>,
    stt_tx: Sender<Vec<i16>>,
    server_control: ServerControl,
    recorder: Arc<AudioRecorder>,
    logs: Vec<String>,
    listening: bool,
    stt_busy: bool,
    tts_busy: bool,
    last_transcript: String,
    last_spoken: String,
    endpoint: String,
    persona_input: String,
    message_input: String,
    hotkey_draft: String,
    server_port_draft: u16,
    tts_bridge_enabled_draft: bool,
    show_endpoint_text_draft: bool,
    stt_language_draft: String,
    stt_model_draft: String,
    stt_model_by_provider_draft: HashMap<String, String>,
    stt_provider_draft: Provider,
    tts_provider_draft: Provider,
    tts_voice_draft: String,
    tts_voice_by_provider_draft: HashMap<String, String>,
    xai_style_draft: String,
    last_save_status: Option<(String, Instant)>,
    last_applied_height: f32,
}

impl Push2TypeApp {
    pub fn new(
        config: Arc<Mutex<AppConfig>>,
        events: Receiver<AppEvent>,
        tts_tx: Sender<TtsRequest>,
        stt_tx: Sender<Vec<i16>>,
        recorder: Arc<AudioRecorder>,
        server_control: ServerControl,
    ) -> Self {
        let cfg = config.lock().expect("config lock").clone();
        let initial_stt_model = cfg.stt_model_for(&cfg.stt_provider);
        let mut tts_voice_by_provider_draft = HashMap::new();
        tts_voice_by_provider_draft.insert("xai".to_string(), cfg.xai_voice.clone());
        tts_voice_by_provider_draft.insert("openai".to_string(), cfg.openai_voice.clone());
        tts_voice_by_provider_draft.insert("groq".to_string(), cfg.groq_voice.clone());
        let tts_voice_draft = tts_voice_by_provider_draft
            .get(provider_label(cfg.tts_provider))
            .cloned()
            .unwrap_or_else(|| cfg.xai_voice.clone());
        Self {
            config,
            events,
            tts_tx,
            stt_tx,
            server_control,
            recorder,
            logs: vec!["Push2Type Rust satellite started.".to_string()],
            listening: false,
            stt_busy: false,
            tts_busy: false,
            last_transcript: String::new(),
            last_spoken: String::new(),
            endpoint: if cfg.tts_bridge_enabled {
                format!("http://127.0.0.1:{}/speak", cfg.server_port)
            } else {
                "Disabled".to_string()
            },
            persona_input: "codex".to_string(),
            message_input: "The quick brown fox jumped over the lazy dog.".to_string(),
            hotkey_draft: cfg.hotkey,
            server_port_draft: cfg.server_port,
            tts_bridge_enabled_draft: cfg.tts_bridge_enabled,
            show_endpoint_text_draft: cfg.show_endpoint_text,
            stt_language_draft: cfg.stt_language,
            stt_model_draft: initial_stt_model,
            stt_model_by_provider_draft: cfg.stt_model_by_provider,
            stt_provider_draft: cfg.stt_provider,
            tts_provider_draft: cfg.tts_provider,
            tts_voice_draft,
            tts_voice_by_provider_draft,
            xai_style_draft: cfg.xai_tts_style,
            last_save_status: None,
            last_applied_height: 280.0,
        }
    }

    fn drain_events(&mut self) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                AppEvent::Info(msg) => self.logs.push(format!("INFO: {msg}")),
                AppEvent::Warning(msg) => self.logs.push(format!("WARN: {msg}")),
                AppEvent::Error(msg) => self.logs.push(format!("ERR: {msg}")),
                AppEvent::Listening(v) => self.listening = v,
                AppEvent::SttBusy(v) => self.stt_busy = v,
                AppEvent::TtsBusy(v) => self.tts_busy = v,
                AppEvent::LastTranscript(text) => self.last_transcript = text,
                AppEvent::LastSpoken(text) => self.last_spoken = text,
                AppEvent::ServerOnline(addr) => self.endpoint = addr,
                AppEvent::ServerOffline => self.endpoint = "Disabled".to_string(),
            }
        }
        if self.logs.len() > 300 {
            let keep = self.logs.split_off(self.logs.len().saturating_sub(300));
            self.logs = keep;
        }
    }
}

impl eframe::App for Push2TypeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_events();
        ctx.request_repaint_after(Duration::from_millis(120));

        let mut save_main = false;
        let mut content_height = 280.0f32;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Push2Type Satellite");
            ui.label("Mic -> STT -> Paste");
            ui.label("HTTP -> TTS -> Speakers");
            ui.monospace(format!("Endpoint: {}", self.endpoint));

            ui.horizontal(|ui| {
                let mic = if self.listening {
                    "Mic: Listening"
                } else {
                    "Mic: Idle"
                };
                let stt = if self.stt_busy {
                    "STT: Busy"
                } else {
                    "STT: Idle"
                };
                let tts = if self.tts_busy {
                    "TTS: Busy"
                } else {
                    "TTS: Idle"
                };
                ui.monospace(mic);
                ui.separator();
                ui.monospace(stt);
                ui.separator();
                ui.monospace(tts);
            });

            ui.separator();
            egui::CollapsingHeader::new("Operations")
                .id_salt("section_ops")
                .default_open(true)
                .show(ui, |ui| {
                    ui.monospace(format!(
                        "STT: {}/{}",
                        provider_label(self.stt_provider_draft),
                        self.stt_model_draft
                    ));
                    ui.monospace(format!("TTS: {}", provider_label(self.tts_provider_draft)));
                    ui.label(format!("Last Transcript: {}", self.last_transcript));
                });

            egui::CollapsingHeader::new("Advanced")
                .id_salt("section_advanced")
                .default_open(false)
                .show(ui, |ui| {
                    egui::CollapsingHeader::new("Configuration")
                        .id_salt("section_config")
                        .default_open(false)
                        .show(ui, |ui| {
                            egui::CollapsingHeader::new("Input Capture")
                                .id_salt("cfg_input_capture")
                                .default_open(false)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label("Hotkey");
                                        ui.text_edit_singleline(&mut self.hotkey_draft);
                                    });
                                    ui.label("Hotkey changes require app restart.");
                                });

                            egui::CollapsingHeader::new("Speech To Text")
                                .id_salt("cfg_stt")
                                .default_open(false)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let old_stt_provider = self.stt_provider_draft;
                                        ui.label("Provider");
                                        egui::ComboBox::from_id_salt("stt_provider")
                                            .selected_text(provider_label(self.stt_provider_draft))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut self.stt_provider_draft,
                                                    Provider::Groq,
                                                    "groq",
                                                );
                                                ui.selectable_value(
                                                    &mut self.stt_provider_draft,
                                                    Provider::OpenAi,
                                                    "openai",
                                                );
                                            });
                                        if self.stt_provider_draft != old_stt_provider {
                                            self.stt_model_draft = self
                                                .stt_model_by_provider_draft
                                                .get(provider_label(self.stt_provider_draft))
                                                .cloned()
                                                .unwrap_or_else(|| self.stt_model_draft.clone());
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Model");
                                        let models = {
                                            let cfg = self.config.lock().expect("config lock");
                                            cfg.stt_available_models(self.stt_provider_draft)
                                        };
                                        egui::ComboBox::from_id_salt("stt_model")
                                            .selected_text(self.stt_model_draft.clone())
                                            .show_ui(ui, |ui| {
                                                for model in models {
                                                    ui.selectable_value(
                                                        &mut self.stt_model_draft,
                                                        model.clone(),
                                                        model,
                                                    );
                                                }
                                            });
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Language");
                                        ui.text_edit_singleline(&mut self.stt_language_draft);
                                        ui.label("example: en");
                                    });
                                });

                            egui::CollapsingHeader::new("Text To Speech + Voice Bridge")
                                .id_salt("cfg_tts_bridge")
                                .default_open(false)
                                .show(ui, |ui| {
                                    if ui
                                        .checkbox(
                                            &mut self.tts_bridge_enabled_draft,
                                            "Enable internal TTS bridge server",
                                        )
                                        .changed()
                                    {
                                        self.server_control
                                            .set_enabled(self.tts_bridge_enabled_draft);
                                        if !self.tts_bridge_enabled_draft {
                                            self.endpoint = "Disabled".to_string();
                                        }
                                    }
                                    ui.horizontal(|ui| {
                                        let old_tts_provider = self.tts_provider_draft;
                                        ui.label("TTS Provider");
                                        egui::ComboBox::from_id_salt("tts_provider")
                                            .selected_text(provider_label(self.tts_provider_draft))
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut self.tts_provider_draft,
                                                    Provider::Xai,
                                                    "xai",
                                                );
                                                ui.selectable_value(
                                                    &mut self.tts_provider_draft,
                                                    Provider::OpenAi,
                                                    "openai",
                                                );
                                                ui.selectable_value(
                                                    &mut self.tts_provider_draft,
                                                    Provider::Groq,
                                                    "groq",
                                                );
                                            });
                                        if self.tts_provider_draft != old_tts_provider {
                                            self.tts_voice_draft = self
                                                .tts_voice_by_provider_draft
                                                .get(provider_label(self.tts_provider_draft))
                                                .cloned()
                                                .unwrap_or_else(|| self.tts_voice_draft.clone());
                                        }
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("Voice");
                                        egui::ComboBox::from_id_salt("tts_voice")
                                            .selected_text(self.tts_voice_draft.clone())
                                            .show_ui(ui, |ui| {
                                                for voice in
                                                    tts_voices_for_provider(self.tts_provider_draft)
                                                {
                                                    let v = voice.to_string();
                                                    ui.selectable_value(
                                                        &mut self.tts_voice_draft,
                                                        v.clone(),
                                                        v,
                                                    );
                                                }
                                            });
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label("HTTP Port");
                                        ui.add(
                                            egui::DragValue::new(&mut self.server_port_draft)
                                                .range(1025..=65535),
                                        );
                                    });
                                    ui.checkbox(
                                        &mut self.show_endpoint_text_draft,
                                        "Show endpoint text in UI",
                                    );
                                    ui.horizontal(|ui| {
                                        ui.label("xAI Delivery Style");
                                        ui.add_enabled_ui(
                                            self.tts_provider_draft == Provider::Xai,
                                            |ui| {
                                                ui.text_edit_singleline(&mut self.xai_style_draft);
                                            },
                                        );
                                    });
                                    if self.tts_provider_draft != Provider::Xai {
                                        ui.small(
                                            "Only xAI realtime currently supports style prompting.",
                                        );
                                    }
                                });

                            if ui.button("Save Configuration").clicked() {
                                save_main = true;
                            }
                        });

                    egui::CollapsingHeader::new("Tools")
                        .id_salt("section_tools")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if ui.button("Manual Capture + STT").clicked() {
                                    let recorder = self.recorder.clone();
                                    let stt_tx = self.stt_tx.clone();
                                    std::thread::spawn(move || {
                                        recorder.start_capture();
                                        std::thread::sleep(Duration::from_millis(1300));
                                        let audio = recorder.stop_capture();
                                        if !audio.is_empty() {
                                            let _ = stt_tx.send(audio);
                                        }
                                    });
                                }
                                ui.label("Records ~1.3s then transcribes.");
                            });
                            ui.separator();
                            ui.label("Voice test:");
                            ui.horizontal(|ui| {
                                ui.label("Persona");
                                ui.text_edit_singleline(&mut self.persona_input);
                            });
                            ui.text_edit_singleline(&mut self.message_input);
                            if ui.button("Speak Test").clicked() {
                                let req = TtsRequest {
                                    speak: SpeakRequest {
                                        message: self.message_input.clone(),
                                        persona: Some(self.persona_input.clone()),
                                        voice: Some(self.tts_voice_draft.clone()),
                                        provider: Some(self.tts_provider_draft),
                                        show_text: Some(true),
                                        style: Some(self.xai_style_draft.clone()),
                                    },
                                };
                                let _ = self.tts_tx.send(req);
                            }
                        });

                    egui::CollapsingHeader::new("Logs")
                        .id_salt("section_logs")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.label(format!("Last Spoken: {}", self.last_spoken));
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .max_height(140.0)
                                .show(ui, |ui| {
                                    for line in self.logs.iter().rev().take(80) {
                                        ui.monospace(line);
                                    }
                                });
                        });
                });

            if let Some((status, when)) = &self.last_save_status {
                if when.elapsed() < Duration::from_secs(4) {
                    ui.label(status);
                }
            }
            // Measure only used content height; min_rect can track available panel size and cause growth loops.
            let used_height = ui.cursor().min.y - ui.min_rect().top();
            content_height = used_height.max(120.0);
        });

        let current_width = ctx.input(|i| i.screen_rect().width()).max(420.0);
        let target_height = (content_height + 40.0).clamp(240.0, 900.0);
        if (target_height - self.last_applied_height).abs() > 6.0 {
            self.last_applied_height = target_height;
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                current_width,
                target_height,
            )));
        }

        if save_main {
            let runtime_port = self.server_port_draft;
            let runtime_enabled = self.tts_bridge_enabled_draft;
            let mut cfg = self.config.lock().expect("config lock");
            cfg.hotkey = self.hotkey_draft.clone();
            cfg.server_port = runtime_port;
            cfg.tts_bridge_enabled = runtime_enabled;
            cfg.show_endpoint_text = self.show_endpoint_text_draft;
            cfg.stt_language = self.stt_language_draft.clone();
            self.stt_model_by_provider_draft.insert(
                provider_label(self.stt_provider_draft).to_string(),
                self.stt_model_draft.clone(),
            );
            cfg.stt_model_by_provider = self.stt_model_by_provider_draft.clone();
            cfg.set_stt_model_for(self.stt_provider_draft, self.stt_model_draft.clone());
            cfg.stt_provider = self.stt_provider_draft;
            cfg.tts_provider = self.tts_provider_draft;
            self.tts_voice_by_provider_draft.insert(
                provider_label(self.tts_provider_draft).to_string(),
                self.tts_voice_draft.clone(),
            );
            cfg.xai_voice = self
                .tts_voice_by_provider_draft
                .get("xai")
                .cloned()
                .unwrap_or_else(|| cfg.xai_voice.clone());
            cfg.openai_voice = self
                .tts_voice_by_provider_draft
                .get("openai")
                .cloned()
                .unwrap_or_else(|| cfg.openai_voice.clone());
            cfg.groq_voice = self
                .tts_voice_by_provider_draft
                .get("groq")
                .cloned()
                .unwrap_or_else(|| cfg.groq_voice.clone());
            cfg.xai_tts_style = self.xai_style_draft.clone();
            let save_res = cfg.save();
            self.last_save_status = Some(match save_res {
                Ok(_) => ("Saved config.".to_string(), Instant::now()),
                Err(e) => (format!("Save failed: {e}"), Instant::now()),
            });
            drop(cfg);
            self.server_control.set_port(runtime_port);
            self.server_control.set_enabled(runtime_enabled);
            if !runtime_enabled {
                self.endpoint = "Disabled".to_string();
            }
        }
    }
}

fn provider_label(provider: Provider) -> &'static str {
    match provider {
        Provider::Groq => "groq",
        Provider::OpenAi => "openai",
        Provider::Xai => "xai",
    }
}

fn tts_voices_for_provider(provider: Provider) -> Vec<&'static str> {
    match provider {
        Provider::Xai => vec!["ara", "rex", "sal", "eve", "leo"],
        Provider::OpenAi => vec![
            "alloy", "ash", "ballad", "coral", "echo", "fable", "nova", "onyx", "sage", "shimmer",
            "verse", "marin", "cedar",
        ],
        Provider::Groq => vec!["autumn", "diana", "hannah", "austin", "daniel", "troy"],
    }
}
