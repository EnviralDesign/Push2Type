# -*- mode: python ; coding: utf-8 -*-

import os
from glob import glob
from pathlib import Path
import whisper  # Use the installed whisper module

block_cipher = None

# Locate the installed whisper package directory.
whisper_pkg_path = Path(whisper.__file__).parent
whisper_assets_dir = whisper_pkg_path / "assets"

# Prepare the datas list. We always include config.py.
datas = [
    ('config.py', '.'),
]

# If the assets directory exists in the installed whisper package, include its files.
if whisper_assets_dir.exists():
    for file in glob(os.path.join(str(whisper_assets_dir), "*")):
        # Copy each asset file to the destination folder: 
        # "whisper/assets" (so that in the final build, the assets
        # will appear under _internal/whisper/assets as Whisper expects)
        datas.append((file, os.path.join("whisper", "assets")))
else:
    print("Warning: whisper assets directory not found at", whisper_assets_dir)

a = Analysis(
    ['main.py'],
    pathex=[],
    binaries=[],
    datas=datas,
    hiddenimports=[
        # Include any modules that PyInstaller might miss automatically.
        'global_hotkeys',
        'pyaudio',
        'speech_recognition',
        'torch',
        'torchvision',
        'torchaudio',
        'pyautogui',
        'pyperclip',
        'packaging',
        'packaging.requirements'
    ],
    hookspath=[],
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='MyApp',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=True,
    console=False,  # Change to True if you want a console window.
)

coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=True,
    upx_exclude=[],
    name='MyApp'
)