!macro NSIS_HOOK_POSTINSTALL
  nsExec::ExecToLog 'powershell -NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -Command "$p=Join-Path $env:LOCALAPPDATA ''LibreRTC\client\profile.json''; if (Test-Path -LiteralPath $p) { $j=Get-Content -LiteralPath $p -Raw | ConvertFrom-Json; if ($j.profile) { $j.profile.welcome_dismissed=$false } else { $j | Add-Member -NotePropertyName welcome_dismissed -NotePropertyValue $false -Force }; $j | ConvertTo-Json -Depth 32 | Set-Content -LiteralPath $p -Encoding UTF8 }"'
!macroend
