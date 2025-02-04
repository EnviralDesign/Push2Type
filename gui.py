import tkinter as tk
from tkinter import ttk
import config
import threading

class TranscriptionApp(tk.Tk):
    def __init__(self, update_transcription_callback=None, update_log_callback=None,
                 model_change_callback=None, stop_callback=None, *args, **kwargs):
        """
        Initializes the Tkinter-based GUI for live transcription and logging.
        
        Args:
            update_transcription_callback (Callable, optional): External callback for transcription updates.
            update_log_callback (Callable, optional): External callback for logging messages.
            model_change_callback (Callable, optional): Callback function to handle model change.
            stop_callback (Callable, optional): Callback for additional cleanup when stopping.
        """
        super().__init__(*args, **kwargs)
        self.title("Push2Type")
        self.geometry(config.COMPACT_GEOMETRY)  # Fixed, compact geometry e.g., "400x60"
        self.wm_attributes("-topmost", config.ALWAYS_ON_TOP)  # Always on top

        # Main container frame using horizontal layout.
        self.main_frame = tk.Frame(self)
        self.main_frame.pack(fill=tk.BOTH, expand=True, padx=5, pady=5)
        
        # Configuration Frame (left side) for model dropdown and GPU checkbox.
        self.config_frame = tk.Frame(self.main_frame)
        self.config_frame.pack(side=tk.LEFT, padx=5)
        
        # Model Selector Dropdown
        self.model_var = tk.StringVar(self)
        self.model_var.set(config.DEFAULT_MODEL)
        self.model_options = ["tiny", "tiny.en", "base", "base.en", "small", 
                              "small.en", "medium", "medium.en", "large", "turbo"]
        self.model_dropdown = ttk.Combobox(self.config_frame, textvariable=self.model_var, 
                                           values=self.model_options, width=10)
        self.model_dropdown.pack(side=tk.LEFT, padx=5)
        self.model_dropdown.bind("<<ComboboxSelected>>", self.on_model_change)
        
        # GPU Checkbox
        self.gpu_var = tk.BooleanVar(value=True)
        self.gpu_checkbox = tk.Checkbutton(self.config_frame, text="GPU", 
                                           variable=self.gpu_var, command=self.on_gpu_toggle)
        self.gpu_checkbox.pack(side=tk.LEFT, padx=5)
        
        # Status Frame (right side) for displaying the current state.
        self.status_frame = tk.Frame(self.main_frame)
        self.status_frame.pack(side=tk.RIGHT, padx=5)
        # The status label is given a fixed width and right anchor so its size stays constant.
        self.status_label = tk.Label(self.status_frame, text="Idle", font=("Helvetica", 14),
                                     width=20, anchor="e")
        self.status_label.pack(side=tk.RIGHT, padx=5)
        
        # Callback functions for external events.
        self.update_transcription_callback = update_transcription_callback
        self.update_log_callback = update_log_callback
        self.model_change_callback = model_change_callback
        self.stop_callback = stop_callback
        
        # State variable.
        self.push_to_talk_active = False
        
        # Variables for spinner animation.
        self.spinner_running = False
        self.spinner_chars = ["|", "/", "-", "\\"]
        self.current_spinner_index = 0

    def on_gpu_toggle(self):
        # Indicate loading on UI.
        self.status_label.config(text="Loading...")
        self.update()
        if self.model_change_callback:
            # Spawn a thread that calls the model_change_callback and updates the status when done.
            def load_model_and_update():
                self.model_change_callback(self.model_var.get())
                self.after(0, lambda: self.status_label.config(text="Idle"))
            threading.Thread(target=load_model_and_update, daemon=True).start()
            
    def on_model_change(self, event):
        # Indicate loading on UI before triggering the reload.
        self.status_label.config(text="Loading...")
        self.update()
        if self.model_change_callback:
            def load_model_and_update():
                self.model_change_callback(self.model_var.get())
                self.after(0, lambda: self.status_label.config(text="Idle"))
            threading.Thread(target=load_model_and_update, daemon=True).start()
    
    def expand_ui(self):
        # In the compact UI, "expanding" changes the status and starts spinner.
        self.status_label.config(text="Listening...")
        self.start_spinner()
        self.update()

    def contract_ui(self):
        # Stop spinner and revert status.
        self.stop_spinner()
        self.status_label.config(text="Idle")
        self.update()
    
    def start_spinner(self):
        self.spinner_running = True
        self.update_spinner()
    
    def stop_spinner(self):
        self.spinner_running = False
    
    def update_spinner(self):
        if self.spinner_running:
            spinner_char = self.spinner_chars[self.current_spinner_index]
            self.current_spinner_index = (self.current_spinner_index + 1) % len(self.spinner_chars)
            # Combine fixed "Listening..." text with spinner animation.
            self.status_label.config(text=f"Listening... {spinner_char}")
            self.after(200, self.update_spinner)
    
    def update_transcription(self, text: str) -> None:
        # Briefly display the final transcription result then revert to Idle.
        self.status_label.config(text=text)
        self.after(3000, lambda: self.status_label.config(text="Idle"))
    
    def update_log(self, log_message: str) -> None:
        # No log area in the compact UI.
        pass
    
    def start_app(self) -> None:
        self.mainloop() 