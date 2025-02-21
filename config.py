"""
Configuration parameters for the Real-Time Speech-to-Text Transcription Utility.
This module loads configuration from the user config file if available.
"""

from user_config import load_user_config

# Load user configuration
user_config = load_user_config()

# Audio processing configuration
ENERGY_THRESHOLD = user_config["energy_threshold"]
RECORD_TIMEOUT = user_config["record_timeout"]
PHRASE_TIMEOUT = user_config["phrase_timeout"]

# Geometry and display configuration for the compact UI.
COMPACT_GEOMETRY = "400x60"  # A small, narrow widget
ALWAYS_ON_TOP = user_config["always_on_top"]

# Model settings
DEFAULT_MODEL = user_config["model"]
USE_GPU = user_config["use_gpu"]

# Recording configuration
# Delay in milliseconds added after hotkey release before stopping audio capture.
RECORDING_BUFFER_MS = user_config["recording_buffer_ms"]

# Push-to-Talk Configuration
HOTKEY = user_config["hotkey"]
MINIMAL_GEOMETRY = "400x200"   # Contracted mode
EXPANDED_GEOMETRY = "600x200"   # Expanded mode (wider, same height)

# Auto-detect GPU usage; can be overridden via GUI
USE_GPU = True

# When in push-to-talk, the window stays always on top 