# Push2Type: Real-Time Speech-to-Text Transcription Utility

## Rust Rebuild (Preview)

A full Rust rebuild now lives in `rust/push2type-rs` while the Python implementation remains intact as legacy reference.  
The Rust app adds a local AI voice bridge endpoint:
- `POST /speak` on `127.0.0.1:7821` for spoken agent updates
- persona-to-voice mapping support
- xAI voice first with OpenAI fallback

See `rust/push2type-rs/README.md` for setup and endpoint schema.

Push2Type is a straightforward utility for both **LOCAL** and **CLOUD** speech-to-text transcription. Built on OpenAI's Whisper models and a simple Tkinter GUI, it's designed for developers and power users who need reliable, live transcription with flexible processing options.

## What It Does

- **Hybrid Transcription:** Choose between local processing (via Whisper) or cloud-based transcription (via OpenAI's API) on the fly.
- **Multiple Models:**
    - **Local:** Select from various Whisper model variants (e.g., `base`, `small`, `medium`) to balance speed and accuracy.
    - **Cloud:** Utilize powerful cloud models like `whisper-1` and `gpt-4o-transcribe`.
- **Processing Modes:**
    - **GPU:** Use a local CUDA-compatible GPU for high-performance local transcription.
    - **CPU:** Fall back to CPU processing if a GPU is not available.
    - **Cloud:** Offload transcription to the OpenAI API.
- **Push-to-Talk:** Control recording with a configurable hotkey—press to record, release to transcribe. (Default: `Ctrl+Win`)
- **Simple UI:** A minimal, functional interface to switch modes and models effortlessly.

## How It Works

Push2Type listens to your microphone when you hold the push-to-talk hotkey. When you release the key, it processes the captured audio based on your selected mode:
- **Local (CPU/GPU):** The audio is transcribed locally using the selected Whisper model.
- **Cloud:** The audio is sent to the OpenAI API for transcription.

The resulting text is then injected into your active window via a simulated clipboard paste.

## Getting Started

### Requirements

- **Python 3.8+** (Developed and tested with Python 3.12)
- **uv:** A fast Python package installer and resolver.
- **A working microphone**
- **Optional (for GPU mode):** A CUDA-compatible GPU.
- **Optional (for Cloud mode):** An OpenAI API key.

### Quick Setup on Windows

1. **Install `uv`:**
   If you don't have `uv`, install it first. Instructions can be found [here](https://github.com/astral-sh/uv).

2. **Create a Virtual Environment:**
   In your project directory, run:
   ```bash
   uv venv --python 3.12
   ```
   Activate it if you prefer to use regular python commands:
   ```bash
   .venv\Scripts\activate
   ```

3. **Install Dependencies:**
   From the directory holding your .venv folder, run:
   ```bash
   uv pip install -r requirements.txt
   ```

4. **Configure:**
   - On the first run, the application will create a `push2type_config.json` file.
   - To use **Cloud mode**, you must edit this file and add your OpenAI API key:
     ```json
     {
         ...
         "openai_api_key": "YOUR_API_KEY_HERE",
         ...
     }
     ```
   - You can also place your key in a `.env` file in the project root as `OPENAI_API_KEY=...` for local development.

5. **Run Push2Type:**
   Launch the application by running:
   ```bash
   uv run main.py
   ```
   The GUI will open, and you can use your configured hotkey (default: `Ctrl+Win`) to start transcribing.

## Packaging Your Application

If you want to build a standalone executable:

1. **Build It:**
   ```bash
   uv run pyinstaller build.spec
   ```
   The packaged app will be available in the `dist/` directory.

## Contributing

Contributions, bug reports, and suggestions are welcome. If you have ideas for improvements or fixes, feel free to fork the repository and submit a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
