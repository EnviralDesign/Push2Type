$scriptDir = $PSScriptRoot
Start-Process -FilePath "$scriptDir\.venv\Scripts\pythonw.exe" -ArgumentList "$scriptDir\main.py" -WindowStyle Hidden 