import threading
import time
import tkinter as tk
from audio_capture import (
    initialize_microphone,
    start_audio_capture,
    stop_and_flush_audio_capture,
    shutdown_audio,
)
from transcription import load_model, process_audio_data, transcribe_audio
from gui import TranscriptionApp
from logger_setup import setup_logger
from hotkey_handler import register_push_to_talk_hotkey
from injection import inject_text
import config
import numpy as np  # for logging details about the waveform

# Global flag to ensure that we don't restart push-to-talk while transcription is in progress.
transcription_in_progress = False

def app_update_log(message):
    """
    Fallback logging callback that prints messages to the console.
    
    Args:
        message (str): The log message.
    """
    print(message)

def main():
    global transcription_in_progress

    # Setup logger with GUI log callback
    logger = setup_logger(gui_callback=app_update_log)
    logger.info("Logger initialized.")

    # Initialize audio capture (for push-to-talk mode only).
    mic = initialize_microphone()
    logger.info("Microphone initialized.")
    
    # Load the initial transcription model from config.
    initial_model_name = config.DEFAULT_MODEL
    model = load_model(initial_model_name, config.USE_GPU)
    logger.info(f"Whisper model '{initial_model_name}' loaded.")
    current_model = {"model": model}
    
    # Define the model change callback function
    def change_model(new_model_name):
        logger.info(f"Changing model to {new_model_name} at {time.time()}...")
        new_model = load_model(new_model_name, config.USE_GPU)
        current_model["model"] = new_model
        logger.info(f"New model '{new_model_name}' loaded successfully at {time.time()}.")
    
    # Define the stop callback to unload the model and perform housekeeping.
    def unload_model():
        logger.info("Unloading model and performing housekeeping...")
        try:
            if current_model["model"]:
                del current_model["model"]
                current_model["model"] = None
            import torch
            torch.cuda.empty_cache()
            logger.info("Model unloaded and GPU cache emptied successfully.")
        except Exception as e:
            logger.error(f"Error during model unload: {e}")
    
    # Create the GUI application immediately so it is visible.
    app = TranscriptionApp(
        update_log_callback=app_update_log,
        model_change_callback=change_model,
        stop_callback=unload_model,
    )
    
    # Immediately show "Initializing..." to provide visual feedback.
    app.status_label.config(text="Initializing...")
    app.update()
    
    # Load the initial transcription model in a background thread.
    def load_initial_model():
        initial_model_name = config.DEFAULT_MODEL
        model = load_model(initial_model_name, config.USE_GPU)
        current_model["model"] = model
        logger.info(f"Whisper model '{initial_model_name}' loaded.")
        app.after(0, lambda: app.status_label.config(text="Idle"))
    
    threading.Thread(target=load_initial_model, daemon=True).start()

    # Define push-to-talk start and end callbacks with detailed logs.
    def push_to_talk_start():
        global transcription_in_progress
        logger.info(f"[{time.time()}] push_to_talk_start() triggered.")
        if transcription_in_progress:
            logger.info(f"[{time.time()}] Transcription in progress; ignoring new push-to-talk start.")
            return
        logger.info(f"[{time.time()}] Push-to-talk activated.")
        app.push_to_talk_active = True
        start_audio_capture()
        logger.info(f"[{time.time()}] Audio capture started.")
        app.after(0, lambda: (logger.info(f"[{time.time()}] Expanding UI (Recording state)."), app.expand_ui()))
    
    def push_to_talk_end():
        global transcription_in_progress
        logger.info(f"[{time.time()}] push_to_talk_end() triggered.")
        if transcription_in_progress:
            logger.info(f"[{time.time()}] Transcription in progress; ignoring additional push-to-talk end.")
            return
        logger.info(f"[{time.time()}] Push-to-talk deactivated.")
        app.push_to_talk_active = False
        
        # Stop audio capture and flush the buffer.
        audio_data = stop_and_flush_audio_capture()
        logger.info(f"[{time.time()}] Audio capture stopped. Audio data length: {len(audio_data) if audio_data else 0}")
        
        # Update UI status to "Transcribing..." before beginning transcription.
        app.status_label.config(text="Transcribing...")
        app.update()  # Force UI update.
        transcription_in_progress = True
        logger.info(f"[{time.time()}] Transcription state set to True. Starting transcription...")

        def do_transcription():
            global transcription_in_progress
            start_transcription_time = time.time()
            try:
                if audio_data:
                    waveform = process_audio_data(audio_data)
                    # Log details about the waveform (if it's a numpy array)
                    if isinstance(waveform, np.ndarray):
                        logger.info(f"[{time.time()}] Waveform shape: {waveform.shape}, dtype: {waveform.dtype}")
                    else:
                        logger.info(f"[{time.time()}] Waveform type: {type(waveform)}")
                    process_duration = time.time() - start_transcription_time
                    logger.info(f"[{time.time()}] Audio data processed in {process_duration:.3f} seconds.")
                    try:
                        transcription_start_time = time.time()
                        text = transcribe_audio(waveform, current_model["model"])
                        transcription_duration = time.time() - transcription_start_time
                        logger.info(f"[{time.time()}] Transcription raw output: '{text}' (in {transcription_duration:.3f} sec)")
                    except Exception as e:
                        logger.exception(f"[{time.time()}] Error during transcription: {e}")
                        text = ""
                    logger.info(f"[{time.time()}] Final transcription complete with text: '{text}'")
                    app.update_transcription(text)
                    inject_text(text)
                    logger.info(f"[{time.time()}] Injection complete.")
                else:
                    logger.info(f"[{time.time()}] No audio data captured.")
            except Exception as e:
                logger.exception(f"[{time.time()}] Exception in do_transcription: {e}")
            transcription_in_progress = False
            logger.info(f"[{time.time()}] Transcription state set to False. Contracting UI.")
            app.contract_ui()
        
        # Slight delay to allow the status update to register.
        app.after(100, do_transcription)
    
    # Register global hotkey for push-to-talk mode.
    register_push_to_talk_hotkey(push_to_talk_start, push_to_talk_end)
    
    # Start the GUI event loop.
    app.start_app()
    
    # On application exit, clean up audio resources.
    shutdown_audio()
    logger.info("Audio interface shut down.")

if __name__ == "__main__":
    main() 