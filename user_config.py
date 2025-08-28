import os
import sys
import json
import logging
from typing import Dict, Any

logger = logging.getLogger("SpeechToText")

DEFAULT_CONFIG = {
    "hotkey": "ctrl+window",
    "model": "base",
    "use_gpu": True,
    "always_on_top": True,
    "energy_threshold": 1000,
    "record_timeout": 2.0,
    "phrase_timeout": 3.0,
    "recording_buffer_ms": 150,
    "use_cloud_stt": False,
    "cloud_provider": "OpenAI",
    "cloud_model": "gpt-4o-transcribe",
    "openai_api_key": "",
    "openai_base_url": "https://api.openai.com/v1"
}

def get_config_path() -> str:
    """
    Get the path to the user config file.
    For standalone executables, this will be next to the exe.
    For development, this will be in the current directory.
    """
    if getattr(sys, 'frozen', False):
        # We're running in a bundle (PyInstaller)
        return os.path.join(os.path.dirname(sys.executable), 'push2type_config.json')
    else:
        # We're running in a normal Python environment
        return 'push2type_config.json'

def load_user_config() -> Dict[str, Any]:
    """
    Load user configuration from the config file.
    If the file doesn't exist, create it with default values.
    """
    config_path = get_config_path()
    
    try:
        if os.path.exists(config_path):
            with open(config_path, 'r') as f:
                user_config = json.load(f)
                # Merge with defaults to ensure all keys exist
                config = DEFAULT_CONFIG.copy()
                config.update(user_config)
                logger.info(f"Loaded user configuration from {config_path}")
                return config
        else:
            # Create the config file with default values
            save_user_config(DEFAULT_CONFIG)
            logger.info(f"Created new configuration file at {config_path}")
            return DEFAULT_CONFIG.copy()
            
    except Exception as e:
        logger.error(f"Error loading configuration: {e}")
        logger.info("Using default configuration")
        return DEFAULT_CONFIG.copy()

def save_user_config(config: Dict[str, Any]) -> None:
    """
    Save the user configuration to the config file.
    
    Args:
        config: Dictionary containing the configuration to save
    """
    config_path = get_config_path()
    try:
        with open(config_path, 'w') as f:
            json.dump(config, f, indent=4)
        logger.info(f"Saved configuration to {config_path}")
    except Exception as e:
        logger.error(f"Error saving configuration: {e}") 