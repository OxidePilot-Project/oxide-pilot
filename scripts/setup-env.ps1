# Oxide Pilot Environment Setup Script
# This script helps you configure the required environment variables

param(
    [switch]$Interactive,
    [string]$GeminiApiKey = "",
    [string]$QwenAuthUrl = "",
    [string]$QwenTokenUrl = "",
    [string]$QwenClientId = "",
    [string]$QwenClientSecret = ""
)

$envFile = "src-tauri\.env"
$envExampleFile = "src-tauri\.env.example"

Write-Host "🛡️ Oxide Pilot Environment Setup" -ForegroundColor Cyan
Write-Host "=================================" -ForegroundColor Cyan

# Check if .env file already exists
if (Test-Path $envFile) {
    $overwrite = Read-Host "Environment file already exists. Overwrite? (y/N)"
    if ($overwrite -ne "y" -and $overwrite -ne "Y") {
        Write-Host "Setup cancelled." -ForegroundColor Yellow
        exit 0
    }
}

Write-Host "`n📝 Creating environment configuration..." -ForegroundColor Green

# Create .env file content
$envContent = @"
# Oxide Pilot Environment Variables
# Generated on $(Get-Date -Format "yyyy-MM-dd HH:mm:ss")

# =============================================================================
# GOOGLE GEMINI API CONFIGURATION
# =============================================================================
# Get your API key from: https://aistudio.google.com/apikey
GEMINI_API_KEY=$GeminiApiKey

# =============================================================================
# QWEN OAUTH2 DEVICE FLOW CONFIGURATION
# =============================================================================
# Based on OAuth2 Device Authorization Grant (RFC 8628)
# Based on examples from: https://github.com/QwenLM/qwen-code

# Device Authorization Endpoint
QWEN_DEVICE_AUTH_URL=$QwenAuthUrl

# Token Endpoint
QWEN_DEVICE_TOKEN_URL=$QwenTokenUrl

# OAuth2 Client Credentials
QWEN_CLIENT_ID=$QwenClientId
QWEN_CLIENT_SECRET=$QwenClientSecret

# OAuth2 Scopes
QWEN_SCOPE=openid,profile,email

# =============================================================================
# LOCAL LLM CONFIGURATION (Optional)
# =============================================================================
LOCAL_LLM_BASE_URL=http://localhost:11434
LOCAL_LLM_MODEL=llama2
LOCAL_LLM_TIMEOUT=30000

# =============================================================================
# SYSTEM CONFIGURATION
# =============================================================================
OXIDE_LOG_LEVEL=info
OXIDE_DATA_DIR=./data
OXIDE_DEV_MODE=false
"@

if ($Interactive) {
    Write-Host "`n🔑 Google Gemini Configuration" -ForegroundColor Yellow
    if (-not $GeminiApiKey) {
        $GeminiApiKey = Read-Host "Enter your Google Gemini API key (or press Enter to skip)"
    }

    Write-Host "`n🤖 Qwen OAuth2 Configuration" -ForegroundColor Yellow
    Write-Host "You can skip Qwen configuration if you only want to use Google Gemini" -ForegroundColor Gray

    if (-not $QwenAuthUrl) {
        $QwenAuthUrl = Read-Host "Enter Qwen Device Auth URL (or press Enter to skip)"
    }

    if (-not $QwenTokenUrl) {
        $QwenTokenUrl = Read-Host "Enter Qwen Token URL (or press Enter to skip)"
    }

    if (-not $QwenClientId) {
        $QwenClientId = Read-Host "Enter Qwen Client ID (or press Enter to skip)"
    }

    if (-not $QwenClientSecret) {
        $QwenClientSecret = Read-Host "Enter Qwen Client Secret (or press Enter to skip)"
    }

    # Update content with interactive values
    $envContent = $envContent -replace "GEMINI_API_KEY=", "GEMINI_API_KEY=$GeminiApiKey"
    $envContent = $envContent -replace "QWEN_DEVICE_AUTH_URL=", "QWEN_DEVICE_AUTH_URL=$QwenAuthUrl"
    $envContent = $envContent -replace "QWEN_DEVICE_TOKEN_URL=", "QWEN_DEVICE_TOKEN_URL=$QwenTokenUrl"
    $envContent = $envContent -replace "QWEN_CLIENT_ID=", "QWEN_CLIENT_ID=$QwenClientId"
    $envContent = $envContent -replace "QWEN_CLIENT_SECRET=", "QWEN_CLIENT_SECRET=$QwenClientSecret"
}

# Write the .env file
try {
    $envContent | Out-File -FilePath $envFile -Encoding UTF8
    Write-Host "✅ Environment file created successfully: $envFile" -ForegroundColor Green
} catch {
    Write-Host "❌ Error creating environment file: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host "`n📋 Next Steps:" -ForegroundColor Cyan
Write-Host "1. Review the generated $envFile file" -ForegroundColor White
Write-Host "2. Update any placeholder values with your actual credentials" -ForegroundColor White
Write-Host "3. Restart the Oxide Pilot application" -ForegroundColor White
Write-Host "4. See docs/ENVIRONMENT_SETUP.md for detailed configuration instructions" -ForegroundColor White

Write-Host "`n⚠️  Security Reminder:" -ForegroundColor Yellow
Write-Host "- Never commit the .env file to version control" -ForegroundColor White
Write-Host "- Keep your API keys and secrets secure" -ForegroundColor White
Write-Host "- Use environment variables in production deployments" -ForegroundColor White

Write-Host "`n🎉 Setup complete!" -ForegroundColor Green
