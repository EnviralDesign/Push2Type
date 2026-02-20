# Push2Type Agent Contract

## Agent Style

Default persona:
- `codex`

Communication style:
- concise
- technically precise
- low-fluff

## Skills

- Preferred setup: install reusable skills globally in your Codex environment.
- Repository-local reference: `skills/skills.example.md`.

## Voice Bridge Contract

Endpoint:
- `POST http://127.0.0.1:7821/speak`
- `GET http://127.0.0.1:7821/health`

Request body:
- `message` (string, required)
- `persona` (string, optional)
- `provider` (`xai`, `openai`, `groq`, optional)
- `voice` (string, optional)
- `style` (string, optional; xAI only)
- `show_text` (boolean, optional)

Defaults-first behavior:
- send only `message` unless an override is needed
- rely on app configuration for default provider/voice/style

Example minimal payload:
```json
{
  "message": "Build completed. Tests passed."
}
```

Example with override:
```json
{
  "message": "Milestone reached.",
  "provider": "xai",
  "voice": "sal",
  "style": "concise, neutral, technical",
  "show_text": true
}
```

## Provider Notes

xAI voices:
- `ara`, `rex`, `sal`, `eve`, `leo`

OpenAI voices:
- `alloy`, `ash`, `ballad`, `coral`, `echo`, `fable`, `nova`, `onyx`, `sage`, `shimmer`, `verse`, `marin`, `cedar`

Groq voices:
- `autumn`, `diana`, `hannah`, `austin`, `daniel`, `troy`

Constraints:
- Groq TTS message length must be 200 chars or fewer.

## Required Voice Behavior

- Any time the agent sends a text response to the user, it must also send a voice update.
- Voice update must be a natural-language counterpart, not a verbatim copy.
- Keep voice concise and low-jargon to reduce cost and improve clarity.
- If text update is very long, summarize to 1-2 short spoken sentences.
- At the start of chat sessions always probe tts server health at `http://127.0.0.1:7821/health`, if not available silently skip voice updates and stop trying, until/unless asked by the user.

## Few-Shot Pattern

Example 1:
- Text update: `I updated STT provider routing, fixed mono downmix, and cargo check passes.`
- Spoken update: `Quick update: I fixed transcription routing and audio handling, and the build is passing now.`

Example 2:
- Text update: `Hotkey listener now supports modifier-only combos and release-triggered transcription.`
- Spoken update: `Hotkeys are now working the way you wanted: hold to talk, release to transcribe.`

Example 3:
- Text update: `Groq TTS failed with 400 due to invalid voice list; I constrained supported voices and revalidated.`
- Spoken update: `I fixed the Groq voice error by limiting to valid voices. It should work reliably now.`

## Recommended Usage Cadence

Use voice updates for every assistant response:
- quick acknowledgements
- progress updates
- questions to the user
- final completion summaries
