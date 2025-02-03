import whisper
import torch
import numpy as np
import logging

logger = logging.getLogger("SpeechToText")  # Ensure this logger is configured as in logger_setup.py

def load_model(model_name: str, use_gpu: bool) -> object:
    """
    Loads the Whisper model specified by model_name with GPU support if available.
    
    Args:
        model_name (str): The model variant to load (e.g., "medium" or "medium.en").
        use_gpu (bool): Flag indicating whether to use GPU acceleration if available.
    
    Returns:
        object: The loaded Whisper model.
    """
    device = "cuda" if (use_gpu and torch.cuda.is_available()) else "cpu"
    logger.info(f"Loading model '{model_name}' on device: {device}")
    model = whisper.load_model(model_name, device=device)
    return model

def process_audio_data(audio_bytes: bytes) -> np.ndarray:
    """
    Converts raw audio bytes into a normalized NumPy array.
    
    Args:
        audio_bytes (bytes): Raw bytes of audio data.
    
    Returns:
        np.ndarray: A float32 NumPy array representing normalized audio samples.
    """
    audio_np = np.frombuffer(audio_bytes, dtype=np.int16).astype(np.float32) / 32768.0
    return audio_np

def transcribe_audio(audio_array: np.ndarray, model: object) -> str:
    """
    Transcribes the given audio represented as a NumPy array using the Whisper model.
    
    Args:
        audio_array (np.ndarray): Processed audio samples.
        model (object): The Whisper model to use for transcription.
    
    Returns:
        str: The transcribed text.
    """
    result = model.transcribe(audio_array, fp16=False)
    text = result.get('text', '').strip()
    return text 