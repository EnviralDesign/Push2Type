# Real-Time Speech-to-Text Transcription Utility (MVP) with Push-to-Talk Injection

## 1. Project Overview

**Objective:**  
Develop a lightweight Windows utility in Python that continuously captures microphone audio, transcribes speech using OpenAI’s Whisper model, and displays the transcription live in a dedicated GUI. The new “push-to-talk” mode uses a hotkey (for example, Ctrl+Tilde) so that while you hold the key the UI expands and remains “always on top” while transcribing. When you release the hotkey, the complete text block is programmatically pasted into the active field (using a simulated paste operation) and the UI contracts back to its minimal form.

**Key Features:**
- **Live Transcription:** Audio is captured in near-real-time and transcribed using Whisper.
- **Dedicated GUI Window:** Contains a scrollable transcription area and a log area.
- **Hotkey Push-to-Talk Mode:**  
  - **Activation:** When the user holds the designated hotkey, the UI expands, goes “always on top,” and displays live transcription.
  - **Deactivation:** Upon hotkey release, the complete transcription is automatically pasted into the active field and the UI reverts to a minimal state.
- **Model Flexibility & GPU Toggle:** Allows the selection of different Whisper models and toggling GPU usage.
- **Basic Logging:** Critical messages and errors are logged and displayed in-app.
- **Single Microphone Support:** MVP targets one mic input, with room for future expansion.
- **Packageable:** Initially runnable via a Python virtual environment, with plans for a standalone Windows executable.

---

## 2. File & Folder Structure and Required Libraries

### A. File & Folder Structure

A suggested high-level layout:

```
/speech_to_text_app
  ├── main.py                # Main entry point; orchestrates modules and hotkey events.
  ├── gui.py                 # Implements the GUI using Tkinter.
  ├── audio_capture.py       # Handles microphone audio capture and buffering.
  ├── transcription.py       # Processes audio with the Whisper model.
  ├── hotkey_handler.py      # Manages global hotkey registration and push-to-talk events.
  ├── config.py              # Contains configuration variables and parameters.
  ├── logger_setup.py        # Sets up logging (to file and GUI).
  ├── requirements.txt       # Lists Python dependencies.
  └── README.md              # Documentation: setup, usage, troubleshooting.
```

### B. Python Libraries and External Dependencies

- **Core Python Libraries:**  
  - `argparse`, `os`, `datetime`, `queue`, `time`, `logging`
- **Audio Capture & Processing:**  
  - `SpeechRecognition` (plus its dependency `PyAudio`)  
  - `numpy`
- **Machine Learning & Transcription:**  
  - `torch`  
  - `whisper` (installed via GitHub URL)
- **GUI Development:**  
  - `Tkinter` (built into Python)
- **Hotkey & Input Simulation:**  
  - For global hotkey detection: either the [keyboard](https://github.com/boppreh/keyboard) library or [pynput](https://pypi.org/project/pynput/).  
  - For simulating paste: use Windows API (via `pywin32` or `ctypes`) or [pyautogui](https://pypi.org/project/pyautogui/).  
- **Packaging:**  
  - `PyInstaller` (for building a standalone executable)
- **External Tool:**  
  - `ffmpeg` (required by Whisper; to be installed separately)

Include any new libraries in `requirements.txt` as needed.

---

## 3. Functional Requirements and Module Breakdown

This section describes each module’s responsibilities along with key functions, their expected arguments, and outputs.

### A. Configuration Module (`config.py`)

**Purpose:** Centralize configuration parameters.  
**Key Variables (as Python constants):**
- `ENERGY_THRESHOLD`: *int* (e.g., 1000)
- `RECORD_TIMEOUT`: *float* (e.g., 2.0 seconds)
- `PHRASE_TIMEOUT`: *float* (e.g., 3.0 seconds)
- `DEFAULT_MODEL`: *str* (e.g., `"medium"` or `"medium.en"`)
- `USE_GPU`: *bool* (auto-detected, but can be overridden)
- `HOTKEY`: *str* (e.g., `"ctrl+`"` to represent Ctrl+Tilde; adjust per library format)
- `PUSH_TO_TALK_UI_GEOMETRY`: *dict* (defines expanded vs. minimal window sizes)
- `ALWAYS_ON_TOP`: *bool* (flag for UI mode)

### B. Audio Capture Module (`audio_capture.py`)

**Purpose:** Record audio continuously and push raw data to a thread-safe queue.  
**Key Functions:**

1. **`init_microphone()`**  
   - **Args:** None  
   - **Output:** Returns a configured `sr.Microphone` object (sample rate 16000 Hz).

2. **`start_listening(source, recorder, data_queue, record_timeout)`**  
   - **Args:**  
     - `source`: Microphone object from `init_microphone()`.
     - `recorder`: An instance of `sr.Recognizer` (with configured energy threshold).
     - `data_queue`: A thread-safe queue.
     - `record_timeout`: Duration (seconds) for each audio segment.
   - **Output:** Starts a background listener (using `listen_in_background()`) that pushes audio data to `data_queue`.

3. **Callback Function:**  
   - **Signature:** `record_callback(recognizer, audio: sr.AudioData)`  
   - **Operation:** Extracts raw bytes from `audio` and puts them into `data_queue`.

### C. Transcription Module (`transcription.py`)

**Purpose:** Process buffered audio data and transcribe it using the Whisper model.  
**Key Functions:**

1. **`load_model(model_name: str, use_gpu: bool) -> WhisperModel`**  
   - **Args:**  
     - `model_name`: Desired model variant (e.g., `"medium.en"`).
     - `use_gpu`: Flag for GPU usage.
   - **Output:** A loaded Whisper model instance.

2. **`process_audio(audio_data: bytes) -> np.ndarray`**  
   - **Args:**  
     - `audio_data`: Raw audio bytes.
   - **Output:** A normalized NumPy array (dtype `float32`) for transcription.

3. **`transcribe_audio(model, audio_np: np.ndarray) -> str`**  
   - **Args:**  
     - `model`: Loaded Whisper model.
     - `audio_np`: Processed audio data.
   - **Output:** A transcription string.

### D. GUI Module (`gui.py`)

**Purpose:** Provide the user interface for live transcription display, logs, and controls.  
**Key Functions and Components:**

1. **`create_main_window() -> Tk`**  
   - **Args:** None  
   - **Output:** Configured Tkinter main window.

2. **`init_text_area(window: Tk) -> Text`**  
   - **Args:**  
     - `window`: Main Tkinter window.
   - **Output:** A scrollable text widget for transcription display.

3. **`init_log_area(window: Tk) -> Text`**  
   - **Args:**  
     - `window`: Main Tkinter window.
   - **Output:** A text widget for displaying logs.

4. **`init_controls(window: Tk) -> dict`**  
   - **Args:**  
     - `window`: The main window.
   - **Output:** Dictionary of UI controls (buttons, dropdowns for model selection, GPU toggle).

5. **`update_display(text_widget: Text, new_text: str)`**  
   - **Args:**  
     - `text_widget`: Transcription text widget.
     - `new_text`: Updated transcription content.
   - **Output:** Updates the widget display.

6. **`update_log(log_widget: Text, log_message: str)`**  
   - **Args:**  
     - `log_widget`: Log display widget.
     - `log_message`: Log message string.
   - **Output:** Appends log message to widget.

7. **UI State Functions for Push-to-Talk Mode:**
   - **`expand_ui(window: Tk)`**  
     - **Operation:** Adjusts window geometry and sets always-on-top (using `window.wm_attributes("-topmost", True)`) for active push-to-talk mode.
   - **`contract_ui(window: Tk)`**  
     - **Operation:** Restores the minimal window geometry and disables always-on-top.

### E. Hotkey Handler Module (`hotkey_handler.py`)

**Purpose:** Manage global hotkey registration and detect press/release events to control the push-to-talk mode.  
**Key Functions:**

1. **`register_hotkey(on_press_callback: callable, on_release_callback: callable)`**  
   - **Args:**  
     - `on_press_callback`: Function to call when the hotkey is pressed.
     - `on_release_callback`: Function to call when the hotkey is released.
   - **Output:** Registers global hotkey events using a library such as `keyboard` or `pynput`.

2. **Hotkey Event Callbacks:**  
   - **`push_to_talk_start()`**  
     - **Operation:**  
       - Expand the UI (call `expand_ui(window)`).
       - Begin or resume transcription (if not already active).
       - Log the hotkey press event.
   - **`push_to_talk_end()`**  
     - **Operation:**  
       - Finalize the current transcription block.
       - Paste the transcription into the active field:
         - Use a clipboard method (copy text then simulate Ctrl+V) or simulate keystrokes with Windows API.
       - Contract the UI (call `contract_ui(window)`).
       - Log the hotkey release event.

### F. Logging Module (`logger_setup.py`)

**Purpose:** Configure logging to output messages both to a file and to the GUI log widget.  
**Key Functions:**

1. **`setup_logging(gui_callback: callable = None)`**  
   - **Args:**  
     - `gui_callback`: Optional function that accepts log messages for the GUI.
   - **Output:** Configures Python’s logging module with a file handler and, if provided, a GUI handler.

### G. Main Application Module (`main.py`)

**Purpose:** Coordinate initialization, module integration, and event handling.  
**Key Flow & Functions:**

1. **`initialize_app()`**  
   - **Operation:**  
     - Load configuration from `config.py`.
     - Set up logging via `logger_setup.py`.
     - Create the main GUI window using `create_main_window()`, and initialize text and log areas.
     - Initialize the microphone (`init_microphone()`) and start audio capture (`start_listening()`).
     - Load the transcription model (`load_model()`).
     - Register the push-to-talk hotkey by calling `register_hotkey()` from `hotkey_handler.py`, passing in the push-to-talk start and end callbacks.
   - **Output:** Returns key objects (window, data queue, transcription model, etc.).

2. **`processing_loop()`**  
   - **Operation:**  
     - Continuously or periodically check the audio data queue.
     - Process new audio chunks using `process_audio()` and `transcribe_audio()`.
     - Update the transcription display via `update_display()`.
     - This loop can be integrated into the Tkinter event loop using `after()`.

3. **`shutdown_app()`**  
   - **Operation:**  
     - Cleanly terminate background audio listeners and hotkey listeners.
     - Clear temporary assets (such as Whisper cache) if needed.
     - Close the GUI gracefully.

---

## 4. Detailed Push-to-Talk and Injection Flow

1. **Hotkey Press (Push-to-Talk Start):**
   - User holds the designated hotkey (e.g., Ctrl+Tilde).
   - The hotkey handler triggers `push_to_talk_start()`, which:
     - Expands the UI window using `expand_ui(window)` (e.g., increases window size, sets always-on-top).
     - Starts (or resumes) live transcription display in the expanded text area.
     - Logs the start of a push-to-talk session.

2. **Transcription During Hotkey Hold:**
   - Audio is continuously captured, processed, and transcribed.
   - The live transcription is displayed in the expanded, always-on-top window, giving immediate visual feedback.

3. **Hotkey Release (Push-to-Talk End):**
   - When the user releases the hotkey, `push_to_talk_end()` is invoked.
   - This callback:
     - Finalizes the current transcription block.
     - Uses an injection method to paste the complete transcription into the active field:
       - **Clipboard Method:** Temporarily copies the text to the clipboard and simulates a paste command (e.g., Ctrl+V) via a library or Windows API.
       - **Keystroke Simulation:** Uses Windows API functions (such as `SendInput`) or a library like `pyautogui` to simulate key presses.
     - Contracts the UI back to its minimal state using `contract_ui(window)` (e.g., resets window geometry and always-on-top attribute).
     - Logs the paste action and state change.

---

## 5. Deployment & Packaging

- **Environment Setup:**  
  - Create a Python virtual environment.
  - Install dependencies from `requirements.txt` (include libraries for hotkey handling and input simulation).
  - Ensure `ffmpeg` is installed on the system.
  
- **Packaging:**  
  - Use PyInstaller to bundle the application:
    ```bash
    pyinstaller --onefile main.py
    ```
  - Confirm that all dynamic assets (Whisper model caches, etc.) are handled correctly.

- **Documentation:**  
  - Update README.md with detailed setup instructions, hotkey configuration details, usage instructions, and troubleshooting tips.

---

## 6. Testing & Future Considerations

### Testing Strategy

- **Unit Testing:**  
  - Test each module independently (e.g., verify that `process_audio()` normalizes audio correctly).
- **Integration Testing:**  
  - Validate that hotkey events trigger UI expansion, transcription, and eventual text injection.
- **Manual Testing:**  
  - Ensure that the push-to-talk mode works reliably in your target applications (your IDE, etc.), and that text is pasted correctly.
- **Error Handling:**  
  - Simulate issues (e.g., microphone disconnection, failed paste simulation) and confirm that errors are logged and handled gracefully.

### Future Enhancements

- **Refined Injection:**  
  - Explore more robust keystroke simulation or integration with Windows accessibility APIs.
- **Advanced Hotkey Customization:**  
  - Allow the user to configure hotkey combinations via the GUI.
- **Multi-Microphone Support:**  
  - Extend the GUI to allow dynamic microphone selection.
- **UI Enhancements:**  
  - Further refine the UI for a more compact design when not in push-to-talk mode.

---

## 7. Final Notes

The proposed push-to-talk mode with automatic text injection is achievable in Python with libraries such as `keyboard` or `pynput` for global hotkeys and Windows API wrappers (e.g., via `pywin32` or `pyautogui`) for simulating paste events. Although handling global hotkeys and dynamic UI changes always introduces some extra complexity, the design here keeps the logic modular and well-separated from the core transcription pipeline. With thorough testing and careful integration of the hotkey events, you can achieve a robust push-to-talk experience that streamlines your workflow.

This updated spec should provide your development team with a clear, actionable blueprint to implement the feature. If you have any further questions or need additional clarification on any part of the design, feel free to ask!