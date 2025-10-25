#!/bin/bash

# Oxide Pilot Environment Setup Script
# This script helps you configure the required environment variables

set -e

ENV_FILE="src-tauri/.env"
ENV_EXAMPLE_FILE="src-tauri/.env.example"

echo "🛡️  Oxide Pilot Environment Setup"
echo "================================="

# Check if .env file already exists
if [ -f "$ENV_FILE" ]; then
    read -p "Environment file already exists. Overwrite? (y/N): " overwrite
    if [[ ! "$overwrite" =~ ^[Yy]$ ]]; then
        echo "Setup cancelled."
        exit 0
    fi
fi

echo ""
echo "📝 Creating environment configuration..."

# Create .env file content
cat > "$ENV_FILE" << EOF
# Oxide Pilot Environment Variables
# Generated on $(date '+%Y-%m-%d %H:%M:%S')

# =============================================================================
# GOOGLE GEMINI API CONFIGURATION
# =============================================================================
# Get your API key from: https://aistudio.google.com/apikey
GEMINI_API_KEY=

# =============================================================================
# QWEN OAUTH2 DEVICE FLOW CONFIGURATION
# =============================================================================
# Based on OAuth2 Device Authorization Grant (RFC 8628)
# Based on examples from: https://github.com/QwenLM/qwen-code

# Device Authorization Endpoint
QWEN_DEVICE_AUTH_URL=

# Token Endpoint
QWEN_DEVICE_TOKEN_URL=

# OAuth2 Client Credentials
QWEN_CLIENT_ID=
QWEN_CLIENT_SECRET=

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
EOF

echo "✅ Environment file created successfully: $ENV_FILE"

echo ""
echo "📋 Next Steps:"
echo "1. Review the generated $ENV_FILE file"
echo "2. Update any placeholder values with your actual credentials"
echo "3. Restart the Oxide Pilot application"
echo "4. See docs/ENVIRONMENT_SETUP.md for detailed configuration instructions"

echo ""
echo "⚠️  Security Reminder:"
echo "- Never commit the .env file to version control"
echo "- Keep your API keys and secrets secure"
echo "- Use environment variables in production deployments"

echo ""
echo "🎉 Setup complete!"
