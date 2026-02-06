use std::{
    io::Cursor,
    sync::{Arc, Mutex},
    thread,
};

use crossbeam_channel::{Receiver, Sender};
use reqwest::blocking::{Client, multipart};

use crate::{
    app::AppEvent,
    config::{AppConfig, Provider},
    inject::TextInjector,
};

pub fn spawn_stt_worker(
    config: Arc<Mutex<AppConfig>>,
    events: Sender<AppEvent>,
    stt_rx: Receiver<Vec<i16>>,
    injector: Arc<TextInjector>,
    sample_rate: u32,
) {
    thread::spawn(move || {
        let http = Client::new();
        while let Ok(samples) = stt_rx.recv() {
            let _ = events.send(AppEvent::SttBusy(true));
            let seconds_raw = samples.len() as f32 / sample_rate as f32;
            let _ = events.send(AppEvent::Info(format!(
                "stt audio seconds raw={seconds_raw:.2}"
            )));
            let res = transcribe_with_provider(&http, &config, &samples, sample_rate);
            match res {
                Ok((provider, text)) if !text.is_empty() => {
                    let _ = events.send(AppEvent::Info(format!(
                        "stt provider used: {}",
                        provider_name(&provider)
                    )));
                    let _ = events.send(AppEvent::LastTranscript(text.clone()));
                    if let Err(e) = injector.inject_text(&text) {
                        let _ = events.send(AppEvent::Error(format!("inject failed: {e}")));
                    }
                }
                Ok((provider, _)) => {
                    let _ = events.send(AppEvent::Info(format!(
                        "stt produced empty transcript (provider: {})",
                        provider_name(&provider)
                    )));
                }
                Err(e) => {
                    let _ = events.send(AppEvent::Error(format!("stt failed: {e}")));
                }
            }
            let _ = events.send(AppEvent::SttBusy(false));
        }
    });
}

fn transcribe_with_provider(
    client: &Client,
    cfg: &Arc<Mutex<AppConfig>>,
    samples: &[i16],
    sample_rate: u32,
) -> anyhow::Result<(Provider, String)> {
    let current = cfg.lock().expect("config lock").clone();
    let provider = current.stt_provider;
    let key = current
        .stt_key(&provider)
        .ok_or_else(|| anyhow::anyhow!("missing API key for {}", provider_name(&provider)))?;
    let model = current.stt_model_for(&provider);
    let text = transcribe_once(
        client,
        &provider,
        &key,
        &model,
        &current.stt_language,
        samples,
        sample_rate,
    )?;
    Ok((provider, text))
}

fn transcribe_once(
    client: &Client,
    provider: &Provider,
    api_key: &str,
    model: &str,
    language: &str,
    samples: &[i16],
    sample_rate: u32,
) -> anyhow::Result<String> {
    let wav = pcm_to_wav_bytes(samples, sample_rate)?;
    let url = format!(
        "{}/audio/transcriptions",
        AppConfig::stt_base_url(provider).trim_end_matches('/')
    );
    let part = multipart::Part::bytes(wav)
        .file_name("speech.wav")
        .mime_str("audio/wav")?;
    let mut form = multipart::Form::new()
        .text("model", model.to_string())
        .part("file", part);
    if !language.trim().is_empty() {
        form = form.text("language", language.trim().to_string());
    }
    let response = client
        .post(url)
        .bearer_auth(api_key)
        .multipart(form)
        .send()?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("status {}", response.status()));
    }
    let body: serde_json::Value = response.json()?;
    let text = body
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    Ok(text)
}

fn pcm_to_wav_bytes(samples: &[i16], sample_rate: u32) -> anyhow::Result<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(
        &mut cursor,
        hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )?;
    for &sample in samples {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    Ok(cursor.into_inner())
}

fn provider_name(provider: &Provider) -> &'static str {
    match provider {
        Provider::Xai => "xai",
        Provider::OpenAi => "openai",
        Provider::Groq => "groq",
    }
}
