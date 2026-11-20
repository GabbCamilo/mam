param(
    [switch]$SkipBuild,
    [string]$OutputDir = "",
    [string]$PythonRuntimeDir = ""
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$BackendDir = Join-Path $RepoRoot "backend"
$FrontendDir = Join-Path $BackendDir "frontend"
$AiEngineDir = Join-Path $RepoRoot "ai-engine"

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $RepoRoot "dist"
}

$StageDir = Join-Path $OutputDir "MAM-windows-x86_64"
$ZipPath = Join-Path $OutputDir "MAM-windows-x86_64.zip"

if (-not $SkipBuild) {
    Write-Host "==> Compilando frontend React + TypeScript..."
    Push-Location $FrontendDir
    try {
        npm run build
        if ($LASTEXITCODE -ne 0) { throw "Falha ao compilar frontend." }
    } finally {
        Pop-Location
    }

    Write-Host "==> Compilando backend Rust (release)..."
    Push-Location $BackendDir
    try {
        cargo build --release -p app
        if ($LASTEXITCODE -ne 0) { throw "Falha ao compilar backend release." }
    } finally {
        Pop-Location
    }
}

$ExePath = Join-Path $BackendDir "target\release\app.exe"
if (-not (Test-Path $ExePath)) {
    throw "Executável não encontrado em: $ExePath"
}

Write-Host "==> Montando pacote portátil em $StageDir..."
if (Test-Path $StageDir) { Remove-Item $StageDir -Recurse -Force }
New-Item -ItemType Directory -Path $StageDir -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $StageDir "ai-engine") -Force | Out-Null
New-Item -ItemType Directory -Path (Join-Path $StageDir "models") -Force | Out-Null

Copy-Item $ExePath (Join-Path $StageDir "app.exe") -Force
$WebViewDll = Join-Path (Split-Path $ExePath -Parent) "WebView2Loader.dll"
if (Test-Path $WebViewDll) {
    Copy-Item $WebViewDll (Join-Path $StageDir "WebView2Loader.dll") -Force
} else {
    Write-Warning "WebView2Loader.dll não encontrado em $(Split-Path $ExePath -Parent)."
}
Copy-Item (Join-Path $AiEngineDir "mam_ai_engine") (Join-Path $StageDir "ai-engine\mam_ai_engine") -Recurse -Force
Copy-Item (Join-Path $AiEngineDir "requirements.txt") (Join-Path $StageDir "ai-engine\requirements.txt") -Force

if (-not [string]::IsNullOrWhiteSpace($PythonRuntimeDir) -and (Test-Path $PythonRuntimeDir)) {
    Write-Host "==> Incluindo runtime Python portátil de $PythonRuntimeDir..."
    Copy-Item $PythonRuntimeDir (Join-Path $StageDir "python") -Recurse -Force
}

if (Test-Path $ZipPath) { Remove-Item $ZipPath -Force }
Compress-Archive -Path (Join-Path $StageDir "*") -DestinationPath $ZipPath -CompressionLevel Optimal

Write-Host "==> Pacote portátil Windows gerado com sucesso: $ZipPath"
