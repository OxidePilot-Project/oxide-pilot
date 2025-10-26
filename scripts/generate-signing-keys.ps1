# Generate Tauri Signing Keys for Development
# This script generates a private key for signing Tauri updates

param(
    [string]$OutputDir = ".",
    [string]$KeyName = "oxide-pilot-signing-key",
    [switch]$Force
)

Write-Host "🔐 Generating Tauri Signing Keys..." -ForegroundColor Cyan

# Check if tauri CLI is installed
if (-not (Get-Command "tauri" -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Tauri CLI not found. Installing..." -ForegroundColor Red
    cargo install tauri-cli
}

# Create output directory if it doesn't exist
if (-not (Test-Path $OutputDir)) {
    New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
}

$PrivateKeyPath = Join-Path $OutputDir "$KeyName.key"
$PublicKeyPath = Join-Path $OutputDir "$KeyName.pub"

# Check if keys already exist
if ((Test-Path $PrivateKeyPath) -and (-not $Force)) {
    Write-Host "⚠️  Keys already exist at $PrivateKeyPath" -ForegroundColor Yellow
    Write-Host "Use -Force to overwrite existing keys" -ForegroundColor Yellow
    exit 1
}

# Generate signing key pair
Write-Host "🔑 Generating key pair..." -ForegroundColor Green
tauri signer generate -w "$PrivateKeyPath"

if (Test-Path $PrivateKeyPath) {
    Write-Host "✅ Private key generated: $PrivateKeyPath" -ForegroundColor Green

    # Generate public key
    tauri signer sign -k "$PrivateKeyPath" -f "$PublicKeyPath" --empty

    if (Test-Path $PublicKeyPath) {
        Write-Host "✅ Public key generated: $PublicKeyPath" -ForegroundColor Green
    }

    Write-Host ""
    Write-Host "🔒 SECURITY NOTES:" -ForegroundColor Yellow
    Write-Host "1. Keep the private key (.key) secure and never commit it to version control"
    Write-Host "2. The public key (.pub) can be shared and should be included in your app"
    Write-Host "3. Set TAURI_PRIVATE_KEY environment variable to the private key content"
    Write-Host "4. Set TAURI_KEY_PASSWORD if you used a password (recommended for production)"
    Write-Host ""
    Write-Host "📝 Next steps:"
    Write-Host "1. Add the private key to GitHub Secrets as TAURI_PRIVATE_KEY"
    Write-Host "2. Add the key password to GitHub Secrets as TAURI_KEY_PASSWORD (if used)"
    Write-Host "3. Update tauri.conf.json with the public key for update verification"

} else {
    Write-Host "❌ Failed to generate private key" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "🎉 Signing keys generated successfully!" -ForegroundColor Green