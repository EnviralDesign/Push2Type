"""
Configuration parameters for the Real-Time Speech-to-Text Transcription Utility.
This module loads configuration from the user config file if available.
"""
import os
from dotenv import load_dotenv
from user_config import load_user_config

# Load environment variables from .env file
load_dotenv()

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

# Cloud STT settings
USE_CLOUD_STT = user_config["use_cloud_stt"]
CLOUD_PROVIDER = user_config["cloud_provider"]
CLOUD_MODEL = user_config["cloud_model"]
OPENAI_API_KEY = os.getenv("OPENAI_API_KEY", user_config.get("openai_api_key", ""))
OPENAI_BASE_URL = os.getenv("OPENAI_BASE_URL", user_config.get("openai_base_url", "https://api.openai.com/v1"))


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