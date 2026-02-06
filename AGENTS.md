# Push2Type Agent Contract

This repo contains a local voice bridge endpoint for AI agents.

## Persona

Default persona for this repo:
- `codex`

Persona communication style:
- concise
- technically precise
- low-fluff

## Voice Bridge Endpoint

- URL: `http://127.0.0.1:7821/speak`
- Method: `POST`
- Content-Type: `application/json`

Body schema:
- `message` (string, required)
- `persona` (string, optional)
- `voice` (string, optional)
- `provider` (`"xai"`, `"openai"`, or `"groq"`, optional)
- `show_text` (boolean, optional)
- `style` (string, optional)

Example:

```json
{
  "message": "I finished the compile check.",
  "persona": "codex",
  "provider": "xai",
  "show_text": true
}
```
