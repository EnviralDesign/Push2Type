# Skill: push2type-voice-bridge

Use this skill to send spoken status updates to the local Push2Type Rust satellite.

## When to use

- You want to narrate progress updates over voice.
- You want both spoken and text updates for important milestones.
- You need persona-specific voice output.

## Endpoint

- `POST http://127.0.0.1:7821/speak`
- `GET http://127.0.0.1:7821/health`

## Request format

```json
{
  "message": "Status update text",
  "persona": "codex",
  "voice": "rex",
  "provider": "xai",
  "show_text": true,
  "style": "concise and direct"
}
```

## Notes

- `message` is required.
- Choose one provider via `provider`.
- `persona` should align with repo `AGENTS.md`.
