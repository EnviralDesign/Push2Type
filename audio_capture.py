import pyaudio
import threading
import queue
import logging
import time
import wave
import io

logger = logging.getLogger("SpeechToText")

class AudioCapture:
    def __init__(self, audio_queue):
        self.audio_queue = audio_queue
        self.rate = 16000
        self.channels = 1
        self.format = pyaudio.paInt16
        self.chunk = 1024
        
        self._pyaudio_instance = None
        self._stream = None
        self._thread = None
        self._is_capturing = False
        self._temp_buffer = []

        self._initialize_microphone()

    def _initialize_microphone(self):
        try:
            self._pyaudio_instance = pyaudio.PyAudio()
            self._stream = self._pyaudio_instance.open(
                format=self.format,
                channels=self.channels,
                rate=self.rate,
                input=True,
                frames_per_buffer=self.chunk
            )
            self._thread = threading.Thread(target=self._capture_loop, daemon=True)
            self._thread.start()
            logger.info("Audio capture initialized.")
        except Exception as e:
            logger.error(f"Failed to initialize microphone: {e}")
            # Propagate error to the GUI if possible

    def _capture_loop(self):
        while True:
            try:
                data = self._stream.read(self.chunk, exception_on_overflow=False)
                if self._is_capturing:
                    self._temp_buffer.append(data)
            except IOError as e:
                if e.errno == pyaudio.paInputOverflowed:
                    logger.warning("Input overflowed. Dropping frame.")
                else:
                    logger.error(f"Error in audio capture loop: {e}")
                    break
            except Exception as e:
                logger.error(f"An unexpected error occurred in audio capture loop: {e}")
                break
            time.sleep(0.01) # Yield

    def start_capture(self):
        self._temp_buffer = []
        self._is_capturing = True
        logger.info("Started audio capture.")

    def stop_capture(self):
        self._is_capturing = False
        logger.info("Stopped audio capture.")
        
        if not self._temp_buffer:
            return None

        # Combine audio data into a single bytes object
        audio_data = b''.join(self._temp_buffer)
        self._temp_buffer = []
        
        # Return the raw bytes, main.py will handle WAV conversion if needed
        return audio_data

    def is_capturing(self):
        return self._is_capturing

    def shutdown(self):
        logger.info("Shutting down audio capture.")
        if self._stream:
            self._stream.stop_stream()
            self._stream.close()
        if self._pyaudio_instance:
            self._pyaudio_instance.terminate()