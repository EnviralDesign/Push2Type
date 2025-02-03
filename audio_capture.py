"""
audio_capture.py

This module handles manual audio capture for push-to-talk.
It uses PyAudio to continuously capture audio data in a background thread.
Audio data is appended to a global buffer only when "recording" is active.
On stop, the full audio buffer is combined and returned.
"""

import pyaudio
import threading
import time

# Audio format configuration
RATE = 16000            # Sampling rate (Hz)
CHANNELS = 1            # Mono audio
FORMAT = pyaudio.paInt16  # 16-bit int format
CHUNK = 1024            # Number of audio frames per buffer

# Internal global state
_is_recording = False       # Flag indicating when to accumulate audio data
_audio_buffer = []          # List to store incoming audio chunks (bytes)
_audio_thread = None        # Background thread for audio capture
_stream = None              # The current PyAudio stream
_audio_interface = None     # The PyAudio interface instance

def initialize_microphone():
    """
    Initializes the PyAudio stream and starts the background audio capture thread.
    
    Returns:
        stream: The initialized PyAudio input stream.
    """
    global _audio_interface, _stream, _audio_thread
    _audio_interface = pyaudio.PyAudio()
    
    _stream = _audio_interface.open(
        format=FORMAT,
        channels=CHANNELS,
        rate=RATE,
        input=True,
        frames_per_buffer=CHUNK
    )
    
    # Start a background thread to capture audio continuously.
    _audio_thread = threading.Thread(target=_audio_capture_loop, daemon=True)
    _audio_thread.start()
    print("Audio capture initialized and background thread started.")
    return _stream

def start_audio_capture():
    """
    Begins accumulating audio data.
    This should be called when push-to-talk is activated.
    """
    global _is_recording, _audio_buffer
    _is_recording = True
    _audio_buffer = []  # Clear any previous audio data
    print("Audio capture started: now recording audio.")

def stop_and_flush_audio_capture():
    """
    Stops accumulating audio data, flushes the buffer, and returns the combined audio bytes.
    
    Returns:
        bytes: The concatenated audio data captured during the recording session.
    """
    global _is_recording, _audio_buffer
    _is_recording = False
    final_audio = b"".join(_audio_buffer)
    _audio_buffer = []  # Reset buffer for next recording
    print("Audio capture stopped. Buffer flushed and audio data returned.")
    return final_audio

def _audio_capture_loop():
    """
    Internal function that continuously reads audio from the microphone.
    It appends data to the buffer only when _is_recording is True.
    """
    global _stream, _audio_buffer, _is_recording
    while True:
        try:
            data = _stream.read(CHUNK, exception_on_overflow=False)
            if _is_recording:
                _audio_buffer.append(data)
        except Exception as e:
            print("Error capturing audio:", e)
        time.sleep(0.01)  # Short sleep to yield time to other threads

def shutdown_audio():
    """
    Cleans up the audio stream and PyAudio interface.
    Call this on program exit to release audio resources.
    """
    global _stream, _audio_interface
    if _stream is not None:
        _stream.stop_stream()
        _stream.close()
    if _audio_interface is not None:
        _audio_interface.terminate()
    print("Audio interface shut down.") 