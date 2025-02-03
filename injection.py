import pyautogui
import pyperclip
import time

def inject_text(text: str) -> None:
    """
    Simulates a paste operation into the active window.
    Copies the provided text to the clipboard, then triggers a paste via Ctrl+V.
    
    Args:
        text (str): Text to be injected.
    """
    pyperclip.copy(text)
    time.sleep(0.1)  # Ensure clipboard is updated
    pyautogui.hotkey("ctrl", "v") 