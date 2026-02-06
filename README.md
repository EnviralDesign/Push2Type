# Push2Type Satellite (Rust)

Push2Type is now a Rust-first desktop satellite for:
- push-to-talk speech-to-text paste
- localhost AI-to-user voice delivery

Current implemented flows:
- `Mic -> STT -> Paste` using push-to-talk + active-window paste.
- `HTTP -> TTS -> Speakers` via a lightweight localhost endpoint.
- Persona-aware voice routing (different voices per AI persona).
- Configurable single provider for each pipeline (no fallback chain).

## Run

1. Create `.env` in repo root:
   - `XAI_API_KEY=...`
   - `OPENAI_API_KEY=...`
   - `GROQ_API_KEY=...`
2. Start:
  - `cargo run`
3. Validate:
   - `cargo check`

## Push-to-talk behavior

- This is hold-to-talk, not toggle.
- Press and hold hotkey -> recording starts.
- Release below-threshold combo state -> recording stops and STT runs.
- Default hotkey is `ctrl+shift`.
- Modifier-only combos like `ctrl+shift` are supported.
- `win` combos are often intercepted by Windows, so avoid them for reliability.
- If you change hotkey in UI config, restart app for listener reload.

## Local endpoint

- `POST http://127.0.0.1:7821/speak`
- `GET http://127.0.0.1:7821/health`

Request body:

```json
{
  "message": "Build finished successfully.",
  "persona": "codex",
  "voice": "rex",
  "provider": "xai",
  "show_text": true,
  "style": "confident, concise"
}
```

All optional fields can be omitted except `message`.

Provider model:
- STT default: `openai` (batch `/audio/transcriptions`)
- TTS default: `xai` (realtime websocket voice)
- You can switch STT/TTS provider in the UI and save config.
- STT model list is provider-specific via dropdown (`STT Provider` then `STT Model`).
- UI defaults to a low-footprint operations view with collapsible configuration sections.

TTS provider notes:
- OpenAI `/audio/speech` supports `pcm` output; app decodes and plays directly.
- Groq `/audio/speech` (Orpheus) currently supports `wav` output and has a 200-char input limit.
- Persona mapped voices are validated per provider; invalid mappings auto-fallback to provider default voice.

## Skills

- Keep personal/automation skills installed globally in your Codex environment.
- This repo includes a reference template at `skills/skills.example.md`.

## Config file

On first run, config is created at:
- `%LOCALAPPDATA%/Push2TypeRs/push2type_rs_config.json`

This file controls hotkey, providers, models, server port, and persona-to-voice mapping.
