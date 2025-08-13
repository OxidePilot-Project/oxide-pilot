param(
  [switch]$Run,
  [Alias("Host")][string]$ListenHost = "127.0.0.1",
  [int]$Port = 8765
)

$ErrorActionPreference = "Stop"

Write-Host "== Cognee Sidecar Setup ==" -ForegroundColor Cyan

function Ensure-Tool($name, $cmd, $help) {
  if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
    Write-Error "Falta '$name'. $help"
  }
}

Ensure-Tool "Python" "python" "Instala Python 3.8+ desde https://www.python.org/downloads/windows/ y agrega python al PATH"

$sidecarDir = Join-Path (Get-Location) "cognee-sidecar"
if (-not (Test-Path $sidecarDir)) { Write-Error "No se encontró carpeta 'cognee-sidecar'" }

# 1) Crear venv si no existe
$venvDir = Join-Path $sidecarDir ".venv"
if (-not (Test-Path $venvDir)) {
  Write-Host "Creando entorno virtual..." -ForegroundColor Yellow
  Push-Location $sidecarDir
  try {
    python -m venv .venv
  } finally {
    Pop-Location
  }
}

# 2) Instalar dependencias
Write-Host "Instalando dependencias..." -ForegroundColor Yellow
$pythonExe = Join-Path $venvDir "Scripts/python.exe"
$pipExe = Join-Path $venvDir "Scripts/pip.exe"

Push-Location $sidecarDir
try {
  & $pipExe install -U pip
  & $pipExe install -e .
} finally {
  Pop-Location
}

# 3) Preparar .env
$envFile = Join-Path $sidecarDir ".env"
if (-not (Test-Path $envFile)) {
  Copy-Item (Join-Path $sidecarDir ".env.example") $envFile -Force
}

# Si el entorno trae LLM_API_KEY en variable de entorno, lo inyectamos
if ($env:LLM_API_KEY) {
  (Get-Content $envFile) | ForEach-Object {
    if ($_ -match "^LLM_API_KEY=") { "LLM_API_KEY=$($env:LLM_API_KEY)" } else { $_ }
  } | Set-Content $envFile
}

# 4) Ejecutar uvicorn si se solicita
if ($Run) {
  Write-Host "Lanzando sidecar en http://${ListenHost}:$Port ..." -ForegroundColor Green
  $uvicorn = Join-Path $venvDir "Scripts/uvicorn.exe"
  $uvicornArgs = @("cognee_sidecar.app:app", "--host", $ListenHost, "--port", $Port, "--reload")
  Start-Process -FilePath $uvicorn -ArgumentList $uvicornArgs -WorkingDirectory $sidecarDir
  Write-Host "Sidecar iniciado en segundo plano." -ForegroundColor Green
}
