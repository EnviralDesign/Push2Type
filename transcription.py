import whisper
import torch
import numpy as np
import logging
import openai
from openai import OpenAI
import config
import io

logger = logging.getLogger("SpeechToText")

class LocalWhisperTranscriber:
    def __init__(self, model_name, use_gpu):
        self.model = self._load_model(model_name, use_gpu)

    def _load_model(self, model_name, use_gpu):
        device = "cuda" if (use_gpu and torch.cuda.is_available()) else "cpu"
        logger.info(f"Loading model '{model_name}' on device: {device}")
        return whisper.load_model(model_name, device=device)

    def transcribe(self, audio_array):
        result = self.model.transcribe(audio_array, fp16=False)
        return result.get('text', '').strip()

import wave

class OpenAITranscriber:
    def __init__(self, api_key, base_url, model):
        if not api_key:
            raise ValueError("OpenAI API key is required for cloud transcription.")
        self.client = OpenAI(api_key=api_key, base_url=base_url)
        self.model = model
        logger.info(f"Using OpenAI model: {self.model}")

    def transcribe(self, audio_array):
        # Convert float32 numpy array to 16-bit int bytes
        audio_data_int16 = (audio_array * 32768.0).astype(np.int16)
        
        wav_bytes_io = io.BytesIO()
        with wave.open(wav_bytes_io, 'wb') as wav_file:
            wav_file.setnchannels(1)  # Mono
            wav_file.setsampwidth(2)  # 16-bit
            wav_file.setframerate(16000) # 16kHz
            wav_file.writeframes(audio_data_int16.tobytes())
        
        wav_bytes_io.seek(0)
        
        # Create a file-like object for the API
        audio_file = ("audio.wav", wav_bytes_io, "audio/wav")
        
        try:
            transcript = self.client.audio.transcriptions.create(
                model=self.model,
                file=audio_file
            )
            return transcript.text.strip()
        except openai.APIError as e:
            logger.error(f"OpenAI API error: {e}")
            return f"Error: {e}"


def get_transcriber(use_cloud_stt, model_name, use_gpu):
    if use_cloud_stt:
        return OpenAITranscriber(
            api_key=config.OPENAI_API_KEY,
            base_url=config.OPENAI_BASE_URL,
            model=config.CLOUD_MODEL
        )
    else:
        return LocalWhisperTranscriber(model_name, use_gpu)

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

def transcribe_audio(audio_array: np.ndarray, transcriber: object) -> str:
    """
    Transcribes the given audio represented as a NumPy array using the provided transcriber.
    
    Args:
        audio_array (np.ndarray): Processed audio samples.
        transcriber (object): The transcriber object (local or cloud).
    
    Returns:
        str: The transcribed text.
    """
    return transcriber.transcribe(audio_array)