
import queue
import threading
import logging
import config
from audio_capture import AudioCapture
from transcription import get_transcriber, process_audio_data, transcribe_audio
from injection import inject_text
from hotkey_handler import HotkeyHandler
from gui import TranscriptionApp
import logger_setup
from user_config import save_user_config, load_user_config

# Initialize logger
logger_setup.setup_logger()
logger = logging.getLogger("SpeechToText")

# --- State and Configuration ---
global_state = {
    "running": True,
    "transcriber": None,
    "audio_capture": None,
    "app": None
}

def reload_model(mode, model):
    """Reloads the transcription model based on the selected mode and model."""
    app = global_state.get("app")
    try:
        use_cloud = (mode == "Cloud")
        use_gpu = (mode == "GPU")
        
        global_state["transcriber"] = get_transcriber(use_cloud, model, use_gpu)
        logger.info(f"Switched to mode: {mode}, model: {model}")
        if app: app.set_status("Idle")
    except Exception as e:
        logger.error(f"Failed to load model: {e}")
        if app: app.set_status(f"Error: {e}")

def save_config(new_config):
    """Saves the configuration and reloads necessary components."""
    user_config = load_user_config()
    
    mode = new_config.get("mode", "GPU")
    model = new_config.get("model", "base")

    user_config["use_cloud_stt"] = (mode == "Cloud")
    user_config["use_gpu"] = (mode == "GPU")
    
    if user_config["use_cloud_stt"]:
        user_config["cloud_model"] = model
    else:
        user_config["model"] = model

    save_user_config(user_config)
    
    # Update live config from newly saved user_config
    config.USE_CLOUD_STT = user_config["use_cloud_stt"]
    config.CLOUD_MODEL = user_config["cloud_model"]
    config.DEFAULT_MODEL = user_config["model"]
    config.USE_GPU = user_config["use_gpu"]
    logger.info("Configuration saved.")

def process_and_transcribe(audio_data_bytes):
    """Processes audio data and performs transcription."""
    app = global_state["app"]
    transcriber = global_state["transcriber"]
    if not transcriber:
        logger.error("Transcriber not initialized.")
        if app: app.set_status("Error: No Transcriber")
        return

    try:
        audio_np = process_audio_data(audio_data_bytes)
        text = transcribe_audio(audio_np, transcriber)
        if text:
            logger.info(f"Injecting text: {text}")
            inject_text(text)
        else:
            logger.info("No text detected.")
    except Exception as e:
        logger.error(f"Error during transcription: {e}")
        if app: app.set_status(f"Error: {e}")
    finally:
        if app: app.set_status("Idle")

def start_audio_capture():
    """Starts the audio capture thread."""
    audio_capture = global_state["audio_capture"]
    if audio_capture and not audio_capture.is_capturing():
        audio_capture.start_capture()

def stop_audio_capture_and_transcribe():
    """Stops audio capture and triggers transcription."""
    audio_capture = global_state["audio_capture"]
    if audio_capture and audio_capture.is_capturing():
        audio_data = audio_capture.stop_capture()
        if audio_data:
            threading.Thread(target=process_and_transcribe, args=(audio_data,), daemon=True).start()

def setup_and_run():
    """Initializes and runs the application components."""
    # --- Audio Capture Thread ---
    audio_queue = queue.Queue()
    global_state["audio_capture"] = AudioCapture(audio_queue)

    # --- GUI ---
    app = TranscriptionApp(
        model_change_callback=reload_model,
        save_config_callback=save_config
    )
    global_state["app"] = app

    # --- Initial model load ---
    app.reload_model()

    # --- Hotkey Handler ---
    hotkey_handler = HotkeyHandler(
        on_press=lambda: (app.expand_ui(), start_audio_capture()),
        on_release=lambda: (app.contract_ui(), stop_audio_capture_and_transcribe()),
        hotkey_str=config.HOTKEY
    )
    hotkey_handler.start()

    logger.info("Application started. Press the hotkey to transcribe.")
    app.start_app()  # This will block until the GUI is closed

    # --- Cleanup ---
    logger.info("Application shutting down.")
    global_state["running"] = False
    hotkey_handler.stop()
    if global_state["audio_capture"]:
        global_state["audio_capture"].shutdown()

if __name__ == "__main__":
    setup_and_run()
