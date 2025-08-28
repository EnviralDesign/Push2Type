import logging
from global_hotkeys import register_hotkeys, start_checking_hotkeys, stop_checking_hotkeys, clear_hotkeys
import threading

logger = logging.getLogger("SpeechToText")

class HotkeyHandler:
    def __init__(self, on_press, on_release, hotkey_str):
        self.on_press = on_press
        self.on_release = on_release
        self.hotkey_str = hotkey_str
        self._is_running = False

    def start(self):
        """Registers the hotkey and starts the listener in a separate thread."""
        if self._is_running:
            logger.warning("Hotkey handler is already running.")
            return

        binding_str = self.hotkey_str.replace("ctrl", "control")
        logger.info(f"Registering hotkey: {binding_str}")

        bindings = [
            {
                "hotkey": binding_str,
                "on_press_callback": self.on_press,
                "on_release_callback": self.on_release,
                "actuate_on_partial_release": False,
            }
        ]
        
        try:
            register_hotkeys(bindings)
            self._is_running = True
            # The start_checking_hotkeys function blocks, so it needs its own thread.
            self.listener_thread = threading.Thread(target=start_checking_hotkeys, daemon=True)
            self.listener_thread.start()
            logger.info("Hotkey listener started.")
        except Exception as e:
            logger.error(f"Failed to register or start hotkey listener: {e}")

    def stop(self):
        """Stops the hotkey listener and unregisters the hotkeys."""
        if not self._is_running:
            return
            
        try:
            stop_checking_hotkeys()
            clear_hotkeys()
            self._is_running = False
            logger.info("Hotkey listener stopped and hotkeys cleared.")
        except Exception as e:
            logger.error(f"Error while stopping hotkey listener: {e}")