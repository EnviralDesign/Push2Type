use std::{collections::HashMap, fs, path::PathBuf};

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Provider {
    #[serde(rename = "xai")]
    Xai,
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "groq")]
    Groq,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub hotkey: String,
    pub stt_model: String,
    pub stt_language: String,
    pub stt_provider: Provider,
    pub tts_provider: Provider,
    pub xai_voice: String,
    pub openai_voice: String,
    pub groq_voice: String,
    pub xai_realtime_model: String,
    pub openai_tts_model: String,
    pub groq_tts_model: String,
    pub groq_stt_model: String,
    pub stt_models: HashMap<String, Vec<String>>,
    pub stt_model_by_provider: HashMap<String, String>,
    pub xai_tts_style: String,
    pub server_port: u16,
    pub show_endpoint_text: bool,
    pub persona_voices: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut persona_voices = HashMap::new();
        persona_voices.insert("codex".to_string(), "rex".to_string());
        persona_voices.insert("reviewer".to_string(), "sal".to_string());
        persona_voices.insert("planner".to_string(), "eve".to_string());

        Self {
            hotkey: "ctrl+shift".to_string(),
            stt_model: "gpt-4o-mini-transcribe-2025-12-15".to_string(),
            stt_language: "en".to_string(),
            stt_provider: Provider::OpenAi,
            tts_provider: Provider::Xai,
            xai_voice: "rex".to_string(),
            openai_voice: "alloy".to_string(),
            groq_voice: "troy".to_string(),
            xai_realtime_model: "grok-4-voice".to_string(),
            openai_tts_model: "gpt-4o-mini-tts-2025-12-15".to_string(),
            groq_tts_model: "canopylabs/orpheus-v1-english".to_string(),
            groq_stt_model: "whisper-large-v3-turbo".to_string(),
            stt_models: default_stt_models(),
            stt_model_by_provider: default_stt_model_by_provider(),
            xai_tts_style: "clear, concise, and technically precise".to_string(),
            server_port: 7821,
            show_endpoint_text: true,
            persona_voices,
        }
    }
}

impl AppConfig {
    pub fn load_or_create() -> anyhow::Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("failed reading config {}", path.display()))?;
            let cfg: Self = serde_json::from_str(&content)
                .with_context(|| format!("failed parsing config {}", path.display()))?;
            Ok(cfg)
        } else {
            let cfg = Self::default();
            cfg.save()?;
            Ok(cfg)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path()?;
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json).with_context(|| format!("failed writing {}", path.display()))?;
        Ok(())
    }

    pub fn config_path() -> anyhow::Result<PathBuf> {
        let base = dirs::data_local_dir().context("cannot resolve local data dir")?;
        let dir = base.join("Push2TypeRs");
        fs::create_dir_all(&dir)?;
        Ok(dir.join("push2type_rs_config.json"))
    }

    pub fn stt_key(&self, provider: &Provider) -> Option<String> {
        match provider {
            Provider::Xai => std::env::var("XAI_API_KEY").ok(),
            Provider::OpenAi => std::env::var("OPENAI_API_KEY").ok(),
            Provider::Groq => std::env::var("GROQ_API_KEY").ok(),
        }
    }

    pub fn stt_base_url(provider: &Provider) -> &'static str {
        match provider {
            Provider::Xai => "https://api.x.ai/v1",
            Provider::OpenAi => "https://api.openai.com/v1",
            Provider::Groq => "https://api.groq.com/openai/v1",
        }
    }

    pub fn stt_model_for(&self, provider: &Provider) -> String {
        let key = provider_key(*provider);
        if let Some(model) = self.stt_model_by_provider.get(key) {
            return model.clone();
        }
        match provider {
            Provider::Groq => self.groq_stt_model.clone(),
            Provider::Xai | Provider::OpenAi => self.stt_model.clone(),
        }
    }

    pub fn stt_available_models(&self, provider: Provider) -> Vec<String> {
        let key = provider_key(provider);
        self.stt_models
            .get(key)
            .cloned()
            .unwrap_or_else(|| vec![self.stt_model_for(&provider)])
    }

    pub fn set_stt_model_for(&mut self, provider: Provider, model: String) {
        let key = provider_key(provider).to_string();
        self.stt_model_by_provider.insert(key, model.clone());
        match provider {
            Provider::Groq => self.groq_stt_model = model,
            Provider::Xai | Provider::OpenAi => self.stt_model = model,
        }
    }
}

fn provider_key(provider: Provider) -> &'static str {
    match provider {
        Provider::Xai => "xai",
        Provider::OpenAi => "openai",
        Provider::Groq => "groq",
    }
}

fn default_stt_models() -> HashMap<String, Vec<String>> {
    let mut m = HashMap::new();
    m.insert(
        "openai".to_string(),
        vec![
            "gpt-4o-mini-transcribe-2025-12-15".to_string(),
            "gpt-4o-transcribe".to_string(),
            "gpt-4o-mini-transcribe".to_string(),
            "whisper-1".to_string(),
        ],
    );
    m.insert(
        "groq".to_string(),
        vec![
            "whisper-large-v3-turbo".to_string(),
            "whisper-large-v3".to_string(),
        ],
    );
    m
}

fn default_stt_model_by_provider() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert(
        "openai".to_string(),
        "gpt-4o-mini-transcribe-2025-12-15".to_string(),
    );
    m.insert("groq".to_string(), "whisper-large-v3-turbo".to_string());
    m
}
