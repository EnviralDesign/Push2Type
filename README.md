# Real-Time Speech-to-Text Transcription Utility (MVP)

## Overview

This utility captures audio from a microphone, transcribes it using OpenAI's Whisper model, and displays the live transcription in a dedicated GUI. It is designed for Windows and uses Python with Tkinter for the GUI.

## File Structure

```
/speech_to_text_app
  ├── main.py
  ├── audio_capture.py
  ├── transcription.py
  ├── gui.py
  ├── config.py
  ├── logger_setup.py
  ├── requirements.txt
  └── README.md
```

## Setup Instructions

1. **Create a virtual environment:**
   ```
   python -m venv venv
   venv\Scripts\activate   # On Windows
   ```

2. **Install dependencies:**
   ```
   pip install -r requirements.txt
   ```

3. **Run the Application:**
   ```
   python main.py
   ```

4. **Packaging (Optional):**
   Use PyInstaller to create a standalone executable:
   ```
   pyinstaller --onefile main.py
   ```

## Notes

- Adjust settings in `config.py` as necessary (e.g., energy threshold, model selection).
- Logs are saved to `app.log` and displayed in the GUI.
- For troubleshooting and future enhancements, review the inline documentation in each module. 