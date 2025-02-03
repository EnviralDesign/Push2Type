# Configuration parameters for the Real-Time Speech-to-Text Transcription Utility

ENERGY_THRESHOLD = 1000  # Adjust this depending on ambient noise levels
RECORD_TIMEOUT = 2.0     # Maximum duration (in seconds) for each recorded phrase
PHRASE_TIMEOUT = 3.0     # Duration (in seconds) to wait for phrase completion

# Geometry and display configuration for the compact UI.
COMPACT_GEOMETRY = "400x60"  # A small, narrow widget
ALWAYS_ON_TOP = True

# Model settings
DEFAULT_MODEL = "base"
USE_GPU = True

# Recording configuration
# Delay in milliseconds added after hotkey release before stopping audio capture.
RECORDING_BUFFER_MS = 150

# Push-to-Talk Configuration
HOTKEY = "ctrl+shift"  # Updated hotkey combination for push-to-talk
MINIMAL_GEOMETRY = "400x200"   # Contracted mode
EXPANDED_GEOMETRY = "600x200"   # Expanded mode (wider, same height)

# Auto-detect GPU usage; can be overridden via GUI
USE_GPU = True

# When in push-to-talk, the window stays always on top 