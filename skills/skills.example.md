# Skills Example (Repo Template)

This repository expects skills to be installed globally for your CLI/user profile.

Use this file as a reference for what to keep in repo-local docs:

## Recommended Global Skills

- `push2type-voice-bridge`
  - Purpose: send spoken updates through `http://127.0.0.1:7821/speak`
  - Default usage: send minimal payload (`message` only)
  - Optional overrides: `provider`, `voice`, `style`, `show_text`, `persona`
  - Operating rule: when posting a user text update, also post a concise spoken counterpart

## Few-Shot Mapping (Text -> Voice)

- Text: `I refactored the hotkey listener and fixed transcription clipping.`
  - Voice: `I fixed hotkeys and improved transcription capture.`

- Text: `I removed legacy Python files, promoted Rust to root, and validated with cargo check.`
  - Voice: `The Rust version is now first-class and the project builds cleanly.`

- Text: `TTS endpoint is up, provider override works, and xAI voice config is now constrained to valid voices.`
  - Voice: `Voice delivery is live and provider overrides are working with valid voice options.`

## Example Skill Invocation Payload

```json
{
  "message": "Implementation milestone complete."
}
```

## Notes

- Keep your personal/global skill definitions outside this repository.
- Keep this file as a shareable example for new users cloning the repo.
