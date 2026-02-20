use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use base64::Engine;
use crossbeam_channel::{Receiver, Sender};
use reqwest::blocking::Client;
use rodio::{OutputStream, Sink, buffer::SamplesBuffer};
use serde::{Deserialize, Serialize};
use tungstenite::{Message, client::IntoClientRequest, connect, stream::MaybeTlsStream};

use crate::{
    app::AppEvent,
    config::{AppConfig, Provider},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakRequest {
    pub message: String,
    pub persona: Option<String>,
    pub voice: Option<String>,
    pub provider: Option<Provider>,
    pub show_text: Option<bool>,
    pub style: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TtsRequest {
    pub speak: SpeakRequest,
}

pub fn spawn_tts_worker(
    config: Arc<Mutex<AppConfig>>,
    events: Sender<AppEvent>,
    tts_rx: Receiver<TtsRequest>,
) {
    thread::spawn(move || {
        let http = Client::new();
        while let Ok(req) = tts_rx.recv() {
            let _ = events.send(AppEvent::TtsBusy(true));
            let current = config.lock().expect("config lock").clone();
            let message = req.speak.message.trim().to_string();
            if message.is_empty() {
                let _ = events.send(AppEvent::Warning("empty speak message".to_string()));
                let _ = events.send(AppEvent::TtsBusy(false));
                continue;
            }

            let show_text = req.speak.show_text.unwrap_or(current.show_endpoint_text);
            if show_text {
                let _ = events.send(AppEvent::LastSpoken(message.clone()));
            }

            let provider = req.speak.provider.unwrap_or(current.tts_provider);
            let voice = resolve_voice(&current, &req.speak, provider);

            let result = synthesize_with_provider(
                &http,
                &current,
                &message,
                &voice,
                &req.speak
                    .style
                    .clone()
                    .unwrap_or(current.xai_tts_style.clone()),
                provider,
            );

            match result {
                Ok(pcm) => {
                    let _ = events.send(AppEvent::Info(format!(
                        "tts provider used: {} voice: {}",
                        provider_name(provider),
                        voice
                    )));
                    if let Err(e) = play_pcm_24k_mono(&pcm) {
                        let _ = events.send(AppEvent::Error(format!("audio playback failed: {e}")));
                    }
                }
                Err(e) => {
                    let _ = events.send(AppEvent::Error(format!("tts failed: {e}")));
                }
            }

            let _ = events.send(AppEvent::TtsBusy(false));
        }
    });
}

fn resolve_voice(cfg: &AppConfig, req: &SpeakRequest, provider: Provider) -> String {
    if let Some(v) = &req.voice {
        let candidate = v.to_lowercase();
        return if is_valid_voice(provider, &candidate) {
            candidate
        } else {
            provider_default_voice(cfg, provider)
        };
    }
    if let Some(p) = &req.persona {
        let persona = p.to_lowercase();
        if let Some(mapped) = cfg.persona_voices.get(&persona) {
            let candidate = mapped.to_lowercase();
            return if is_valid_voice(provider, &candidate) {
                candidate
            } else {
                provider_default_voice(cfg, provider)
            };
        }
    }
    provider_default_voice(cfg, provider)
}

fn synthesize_with_provider(
    client: &Client,
    cfg: &AppConfig,
    message: &str,
    voice: &str,
    style: &str,
    provider: Provider,
) -> anyhow::Result<Vec<i16>> {
    match provider {
        Provider::Xai => {
            let key =
                std::env::var("XAI_API_KEY").map_err(|_| anyhow::anyhow!("XAI_API_KEY missing"))?;
            xai_realtime_tts(message, voice, style, &cfg.xai_realtime_model, &key)
        }
        Provider::OpenAi => {
            let key = std::env::var("OPENAI_API_KEY")
                .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY missing"))?;
            openai_tts(
                client,
                "https://api.openai.com/v1/audio/speech",
                message,
                voice,
                &cfg.openai_tts_model,
                &key,
                "pcm",
            )
        }
        Provider::Groq => {
            let key = std::env::var("GROQ_API_KEY")
                .map_err(|_| anyhow::anyhow!("GROQ_API_KEY missing"))?;
            if message.chars().count() > 200 {
                return Err(anyhow::anyhow!(
                    "Groq Orpheus input max is 200 chars; got {}",
                    message.chars().count()
                ));
            }
            openai_tts(
                client,
                "https://api.groq.com/openai/v1/audio/speech",
                message,
                voice,
                &cfg.groq_tts_model,
                &key,
                "wav",
            )
        }
    }
}

fn openai_tts(
    client: &Client,
    url: &str,
    message: &str,
    voice: &str,
    model: &str,
    api_key: &str,
    response_format: &str,
) -> anyhow::Result<Vec<i16>> {
    let body = serde_json::json!({
        "model": model,
        "voice": voice,
        "input": message,
        "response_format": response_format
    });
    let response = client.post(url).bearer_auth(api_key).json(&body).send()?;
    if !response.status().is_success() {
        let status = response.status();
        let body_text = response.text().unwrap_or_else(|_| "<no body>".to_string());
        return Err(anyhow::anyhow!("HTTP {} body: {}", status, body_text));
    }
    let bytes = response.bytes()?;
    match response_format {
        "pcm" => Ok(bytes
            .chunks_exact(2)
            .map(|c| i16::from_le_bytes([c[0], c[1]]))
            .collect()),
        "wav" => decode_wav_to_i16(bytes.as_ref()),
        _ => Err(anyhow::anyhow!(
            "unsupported response_format decode path: {}",
            response_format
        )),
    }
}

fn xai_realtime_tts(
    message: &str,
    voice: &str,
    style: &str,
    model: &str,
    api_key: &str,
) -> anyhow::Result<Vec<i16>> {
    let mut request = format!("wss://api.x.ai/v1/realtime?model={model}").into_client_request()?;
    request.headers_mut().insert(
        "Authorization",
        format!("Bearer {api_key}")
            .parse()
            .map_err(|e| anyhow::anyhow!("{e}"))?,
    );
    let (mut ws, _) = connect(request)?;
    send_session_update(&mut ws, voice, style)?;
    send_message_and_response(&mut ws, message)?;
    read_audio_until_done(&mut ws)
}

fn send_session_update(
    ws: &mut tungstenite::WebSocket<MaybeTlsStream<TcpStream>>,
    voice: &str,
    style: &str,
) -> anyhow::Result<()> {
    let event = serde_json::json!({
        "type": "session.update",
        "session": {
            "instructions": style,
            "voice": normalize_voice_name(voice),
            "turn_detection": null,
            "audio": {
                "output": {
                    "format": {
                        "type": "audio/pcm",
                        "rate": 24000
                    }
                }
            }
        }
    });
    ws.send(Message::Text(event.to_string()))?;
    Ok(())
}

fn send_message_and_response(
    ws: &mut tungstenite::WebSocket<MaybeTlsStream<TcpStream>>,
    message: &str,
) -> anyhow::Result<()> {
    let item = serde_json::json!({
        "type": "conversation.item.create",
        "item": {
            "type": "message",
            "role": "user",
            "content": [{ "type": "input_text", "text": message }]
        }
    });
    ws.send(Message::Text(item.to_string()))?;

    let response = serde_json::json!({
        "type": "response.create",
        "response": {
            "modalities": ["audio"],
            "instructions": "Speak exactly the most recent user message verbatim. No acknowledgements. No added words."
        }
    });
    ws.send(Message::Text(response.to_string()))?;
    Ok(())
}

fn read_audio_until_done(
    ws: &mut tungstenite::WebSocket<MaybeTlsStream<TcpStream>>,
) -> anyhow::Result<Vec<i16>> {
    let start = Instant::now();
    let mut pcm_bytes = Vec::<u8>::new();
    loop {
        if start.elapsed() > Duration::from_secs(20) {
            return Err(anyhow::anyhow!("xAI realtime timed out"));
        }
        let msg = ws.read()?;
        if let Message::Text(text) = msg {
            let value: serde_json::Value = serde_json::from_str(&text)?;
            let event_type = value
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            if event_type == "response.output_audio.delta" || event_type == "response.audio.delta" {
                if let Some(delta) = value.get("delta").and_then(|v| v.as_str()) {
                    let chunk =
                        base64::engine::general_purpose::STANDARD.decode(delta.as_bytes())?;
                    pcm_bytes.extend_from_slice(&chunk);
                }
            }
            if event_type == "response.output_item.done" {
                if let Some(content) = value.pointer("/item/content").and_then(|v| v.as_array()) {
                    for part in content {
                        if let Some(audio) = part.get("audio").and_then(|v| v.as_str()) {
                            let chunk = base64::engine::general_purpose::STANDARD
                                .decode(audio.as_bytes())?;
                            pcm_bytes.extend_from_slice(&chunk);
                        }
                    }
                }
            }
            if event_type == "response.done" {
                break;
            }
            if event_type == "error" {
                return Err(anyhow::anyhow!("xAI realtime returned error: {value}"));
            }
        }
    }
    Ok(pcm_bytes
        .chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect())
}

fn play_pcm_24k_mono(samples: &[i16]) -> anyhow::Result<()> {
    let (_stream, handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&handle)?;
    let source = SamplesBuffer::new(1, 24_000, samples.to_vec());
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

fn normalize_voice_name(raw: &str) -> String {
    let mut chars = raw.chars();
    if let Some(first) = chars.next() {
        format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase())
    } else {
        "Rex".to_string()
    }
}

fn provider_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Xai => "xai",
        Provider::OpenAi => "openai",
        Provider::Groq => "groq",
    }
}

fn provider_default_voice(cfg: &AppConfig, provider: Provider) -> String {
    match provider {
        Provider::Xai => cfg.xai_voice.to_lowercase(),
        Provider::OpenAi => cfg.openai_voice.to_lowercase(),
        Provider::Groq => cfg.groq_voice.to_lowercase(),
    }
}

fn is_valid_voice(provider: Provider, voice: &str) -> bool {
    match provider {
        Provider::OpenAi => matches!(
            voice,
            "alloy"
                | "ash"
                | "ballad"
                | "coral"
                | "echo"
                | "fable"
                | "nova"
                | "onyx"
                | "sage"
                | "shimmer"
                | "verse"
                | "marin"
                | "cedar"
        ),
        Provider::Groq => matches!(
            voice,
            "autumn" | "diana" | "hannah" | "austin" | "daniel" | "troy"
        ),
        Provider::Xai => matches!(voice, "ara" | "rex" | "sal" | "eve" | "leo"),
    }
}

fn decode_wav_to_i16(bytes: &[u8]) -> anyhow::Result<Vec<i16>> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(anyhow::anyhow!("invalid wav header"));
    }

    let mut offset = 12usize;
    let mut audio_format: Option<u16> = None;
    let mut channels: Option<u16> = None;
    let mut bits_per_sample: Option<u16> = None;
    let mut data_slice: Option<&[u8]> = None;

    while offset + 8 <= bytes.len() {
        let chunk_id = &bytes[offset..offset + 4];
        let chunk_size = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]) as usize;
        let chunk_start = offset + 8;
        if chunk_start > bytes.len() {
            break;
        }
        let chunk_end = (chunk_start + chunk_size).min(bytes.len());

        if chunk_id == b"fmt " && chunk_end >= chunk_start + 16 {
            audio_format = Some(u16::from_le_bytes([
                bytes[chunk_start],
                bytes[chunk_start + 1],
            ]));
            channels = Some(u16::from_le_bytes([
                bytes[chunk_start + 2],
                bytes[chunk_start + 3],
            ]));
            bits_per_sample = Some(u16::from_le_bytes([
                bytes[chunk_start + 14],
                bytes[chunk_start + 15],
            ]));
        } else if chunk_id == b"data" {
            data_slice = Some(&bytes[chunk_start..chunk_end]);
            break;
        }

        offset = chunk_start + chunk_size + (chunk_size % 2);
    }

    let fmt = audio_format.ok_or_else(|| anyhow::anyhow!("wav fmt chunk missing"))?;
    let ch = channels.ok_or_else(|| anyhow::anyhow!("wav channels missing"))?;
    let bps = bits_per_sample.ok_or_else(|| anyhow::anyhow!("wav bits_per_sample missing"))?;
    let data = data_slice.ok_or_else(|| anyhow::anyhow!("wav data chunk missing"))?;

    let sample_bytes = (bps / 8) as usize;
    if sample_bytes == 0 {
        return Err(anyhow::anyhow!("invalid wav sample size"));
    }
    let frame_bytes = sample_bytes * ch as usize;
    if frame_bytes == 0 {
        return Err(anyhow::anyhow!("invalid wav frame size"));
    }
    let valid_len = (data.len() / frame_bytes) * frame_bytes;
    let payload = &data[..valid_len];

    let mut out = Vec::with_capacity(payload.len() / frame_bytes);
    for frame in payload.chunks_exact(frame_bytes) {
        let mut sum = 0i32;
        for channel_idx in 0..ch as usize {
            let start = channel_idx * sample_bytes;
            let s = decode_wav_sample_to_i16(fmt, bps, &frame[start..start + sample_bytes])?;
            sum += s as i32;
        }
        out.push((sum / ch as i32) as i16);
    }
    Ok(out)
}

fn decode_wav_sample_to_i16(
    format: u16,
    bits_per_sample: u16,
    bytes: &[u8],
) -> anyhow::Result<i16> {
    match (format, bits_per_sample) {
        (1, 8) => {
            let v = bytes[0] as i32 - 128;
            Ok((v << 8) as i16)
        }
        (1, 16) => Ok(i16::from_le_bytes([bytes[0], bytes[1]])),
        (1, 24) => {
            let b0 = bytes[0] as i32;
            let b1 = (bytes[1] as i32) << 8;
            let b2 = (bytes[2] as i32) << 16;
            let mut v = b0 | b1 | b2;
            if (v & 0x800000) != 0 {
                v |= !0x00FF_FFFF;
            }
            Ok((v >> 8) as i16)
        }
        (1, 32) => {
            let v = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            Ok((v >> 16) as i16)
        }
        (3, 32) => {
            let f = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            Ok((f.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16)
        }
        (3, 64) => {
            let f = f64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]);
            Ok((f.clamp(-1.0, 1.0) * i16::MAX as f64).round() as i16)
        }
        _ => Err(anyhow::anyhow!(
            "unsupported wav format code {} / {} bits",
            format,
            bits_per_sample
        )),
    }
}
