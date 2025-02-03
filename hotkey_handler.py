from global_hotkeys import register_hotkeys, start_checking_hotkeys, clear_hotkeys

def register_push_to_talk_hotkey(on_activate_callback, on_deactivate_callback, hotkey_combination=None):
    """
    Registers a push-to-talk hotkey using the global-hotkeys library.
    
    Args:
        on_activate_callback: Function to call when the hotkey is pressed.
        on_deactivate_callback: Function to call when the hotkey is released.
        hotkey_combination (str, optional): Hotkey combination string. If None, uses config.HOTKEY.
    """
    # Import configuration for fallback hotkey
    from config import HOTKEY
    if hotkey_combination is None:
        hotkey_combination = HOTKEY

    # Ensure the hotkey is in the correct format: replacing 'ctrl' with 'control'
    binding_str = hotkey_combination.replace("ctrl", "control")
    print("Registering push-to-talk hotkey with binding:", binding_str)
    
    # Create a binding dictionary for the global-hotkeys library.
    binding = {
        "hotkey": binding_str,
        "on_press_callback": on_activate_callback,
        "on_release_callback": on_deactivate_callback,
        "actuate_on_partial_release": False,
    }
    
    # Register the binding and start the hotkey listener.
    register_hotkeys([binding])
    start_checking_hotkeys()
    print("Global hotkey listener started.") 