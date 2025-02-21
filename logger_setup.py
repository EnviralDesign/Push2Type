import logging
import os

class GuiLogHandler(logging.Handler):
    def __init__(self, gui_callback):
        """
        Custom logging handler to push log messages to a GUI.
        
        Args:
            gui_callback (Callable): Function to call with log messages.
        """
        super().__init__()
        self.gui_callback = gui_callback
        
    def emit(self, record):
        log_entry = self.format(record)
        self.gui_callback(log_entry)

def setup_logger(gui_callback=None) -> logging.Logger:
    """
    Configures and returns a logger that outputs to a file, console, and optionally a GUI.
    The log file is cleared on startup.
    
    Args:
        gui_callback (Callable, optional): Callback function for GUI log output.
    
    Returns:
        logging.Logger: The configured logger instance.
    """
    # Clear the log file if it exists
    log_file = "app.log"
    if os.path.exists(log_file):
        with open(log_file, 'w') as f:
            f.truncate(0)
    
    logger = logging.getLogger("SpeechToText")
    logger.setLevel(logging.DEBUG)
    formatter = logging.Formatter("%(asctime)s - %(levelname)s - %(message)s")
    
    # File Handler
    fh = logging.FileHandler(log_file)
    fh.setLevel(logging.DEBUG)
    fh.setFormatter(formatter)
    logger.addHandler(fh)
    
    # Console Handler
    ch = logging.StreamHandler()
    ch.setLevel(logging.INFO)
    ch.setFormatter(formatter)
    logger.addHandler(ch)
    
    # GUI Handler (if provided)
    if gui_callback:
        gh = GuiLogHandler(gui_callback)
        gh.setLevel(logging.DEBUG)
        gh.setFormatter(formatter)
        logger.addHandler(gh)
    
    return logger 