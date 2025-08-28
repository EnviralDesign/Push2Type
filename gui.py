import tkinter as tk
from tkinter import ttk, messagebox
import config
import threading

class TranscriptionApp(tk.Tk):
    def __init__(self, model_change_callback=None, save_config_callback=None, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.title("Push2Type")
        self.geometry("400x80") # Adjusted geometry
        self.wm_attributes("-topmost", config.ALWAYS_ON_TOP)

        self.model_change_callback = model_change_callback
        self.save_config_callback = save_config_callback

        # --- Main UI Frame ---
        self.main_frame = tk.Frame(self)
        self.main_frame.pack(fill=tk.BOTH, expand=True, padx=10, pady=10)
        self.main_frame.grid_columnconfigure(1, weight=1)
        self.main_frame.grid_columnconfigure(3, weight=1)

        # --- Mode Dropdown ---
        self.mode_label = tk.Label(self.main_frame, text="Mode:")
        self.mode_label.grid(row=0, column=0, padx=(0, 5))
        self.mode_var = tk.StringVar(self)
        self.mode_options = ["GPU", "CPU", "Cloud"]
        self.mode_dropdown = ttk.Combobox(self.main_frame, textvariable=self.mode_var, 
                                          values=self.mode_options, width=10)
        self.mode_dropdown.grid(row=0, column=1, sticky="ew")
        self.mode_dropdown.bind("<<ComboboxSelected>>", self.on_mode_or_model_change)

        # --- Model Dropdown ---
        self.model_label = tk.Label(self.main_frame, text="Model:")
        self.model_label.grid(row=0, column=2, padx=(10, 5))
        self.model_var = tk.StringVar(self)
        
        self.local_model_options = ["tiny", "tiny.en", "base", "base.en", "small", "small.en", "medium", "medium.en", "large"]
        self.cloud_model_options = ["whisper-1", "gpt-4o-transcribe"]
        
        self.model_dropdown = ttk.Combobox(self.main_frame, textvariable=self.model_var, width=10)
        self.model_dropdown.grid(row=0, column=3, sticky="ew")
        self.model_dropdown.bind("<<ComboboxSelected>>", self.on_mode_or_model_change)

        # --- Status Label ---

        # --- Status Label ---
        self.status_label = tk.Label(self.main_frame, text="Idle", font=("Helvetica", 12), anchor="e")
        self.status_label.grid(row=1, column=0, columnspan=4, sticky="nsew", pady=(10, 0))

        self.set_initial_state()

        self.spinner_running = False
        self.spinner_chars = ['v', '<', '^', '>']
        self.current_spinner_index = 0

    def set_initial_state(self):
        """Sets the initial state of the dropdowns based on config."""
        if config.USE_CLOUD_STT:
            self.mode_var.set("Cloud")
            self.model_dropdown['values'] = self.cloud_model_options
            self.model_var.set("gpt-4o-transcribe") # Default for Cloud
        elif config.USE_GPU:
            self.mode_var.set("GPU")
            self.model_dropdown['values'] = self.local_model_options
            self.model_var.set("small") # Default for GPU
        else:
            self.mode_var.set("CPU")
            self.model_dropdown['values'] = self.local_model_options
            self.model_var.set("small") # Default for CPU

    def on_mode_or_model_change(self, event):
        """Handles changes in mode or model selection."""
        mode = self.mode_var.get()
        current_model = self.model_var.get()

        # Update model dropdown options based on mode
        if mode == "Cloud":
            new_model_options = self.cloud_model_options
        else: # CPU or GPU
            new_model_options = self.local_model_options
        self.model_dropdown['values'] = new_model_options

        # If the current model isn't in the new list, select the new default
        if current_model not in new_model_options:
            if mode == 'Cloud':
                self.model_var.set('gpt-4o-transcribe')
            else: # CPU or GPU
                self.model_var.set('small')
        
        model = self.model_var.get()

        if mode == "Cloud" and not config.OPENAI_API_KEY:
            messagebox.showwarning("API Key Missing", 
                                     "OpenAI API key is not configured. Please set it in the config file.")
            self.set_initial_state() # Revert to previous state
            return

        if self.save_config_callback:
            new_config = {"mode": mode, "model": model}
            self.save_config_callback(new_config)
        
        self.reload_model()

    def reload_model(self):
        """Triggers the model reload in the main thread."""
        self.set_status("Loading...")
        if self.model_change_callback:
            mode = self.mode_var.get()
            model = self.model_var.get()
            threading.Thread(target=self.model_change_callback, args=(mode, model), daemon=True).start()

    def set_status(self, text):
        """Updates the status label from any thread."""
        def update():
            self.status_label.config(text=text)
            self.update_idletasks()
        self.after(0, update)

    def expand_ui(self):
        self.set_status("Listening...")
        self.start_spinner()

    def contract_ui(self):
        self.stop_spinner()
        mode = self.mode_var.get()
        if mode == "Cloud":
            self.set_status("Transcribing (Cloud)...")
        else:
            self.set_status("Transcribing (Local)...")

    def start_spinner(self):
        self.spinner_running = True
        self.update_spinner()

    def stop_spinner(self):
        self.spinner_running = False

    def update_spinner(self):
        if self.spinner_running:
            spinner_char = self.spinner_chars[self.current_spinner_index]
            self.current_spinner_index = (self.current_spinner_index + 1) % len(self.spinner_chars)
            self.set_status(f"Listening... {spinner_char}")
            self.after(200, self.update_spinner)

    def start_app(self) -> None:
        self.mainloop()