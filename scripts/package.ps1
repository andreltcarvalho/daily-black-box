$ErrorActionPreference = 'Stop'
Push-Location (Join-Path $PSScriptRoot '..')
try {
    npm run build:extension
    if ($LASTEXITCODE -ne 0) { throw 'Build da extensão falhou.' }
    cargo build --manifest-path src-tauri/Cargo.toml --release --bins
    if ($LASTEXITCODE -ne 0) { throw 'Build nativo falhou.' }
    npm run tauri -- build --config src-tauri/tauri.bundle.json
    if ($LASTEXITCODE -ne 0) { throw 'Empacotamento Windows falhou.' }
} finally { Pop-Location }
