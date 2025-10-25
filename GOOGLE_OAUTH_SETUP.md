# Google OAuth2 Setup Guide for Oxide Pilot

This guide will help you set up Google OAuth2 authentication for Oxide Pilot to access Google's Gemini AI services.

## 🚀 Quick Setup

### Option 1: Google OAuth2 (Recommended)

OAuth2 provides secure authentication without storing API keys locally.

#### Step 1: Create Google Cloud Project

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select an existing one
3. Enable the **Generative Language API** (Gemini API):
   - Go to "APIs & Services" > "Library"
   - Search for "Generative Language API"
   - Click "Enable"

#### Step 2: Create OAuth2 Credentials

1. Go to "APIs & Services" > "Credentials"
2. Click "Create Credentials" > "OAuth 2.0 Client IDs"
3. If prompted, configure the OAuth consent screen:
   - Choose "External" for user type
   - Fill in required fields (App name, User support email, Developer contact)
   - Add your email to test users
4. For Application type, select **"Desktop application"**
5. Give it a name (e.g., "Oxide Pilot Desktop")
6. Click "Create"

#### Step 3: Configure Environment Variables

1. Copy your Client ID and Client Secret from the credentials page
2. Open `src-tauri/.env` file
3. Update the following variables:

```env
GOOGLE_OAUTH_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_OAUTH_CLIENT_SECRET=your-client-secret
```

#### Step 4: Test Authentication

1. Start Oxide Pilot: `powershell -ExecutionPolicy Bypass -File scripts/oxide-dev.ps1`
2. Select "Google Gemini" as your AI provider
3. Click "🔗 Connect with Google OAuth2"
4. Your browser will open for authentication
5. Grant permissions to Oxide Pilot
6. Return to the app - you should be authenticated!

### Option 2: API Key (Alternative)

If you prefer a simpler setup, you can use an API key instead:

1. Go to [Google AI Studio](https://aistudio.google.com/apikey)
2. Create a new API key
3. In Oxide Pilot, select "Google Gemini"
4. Choose "🔑 API Key" option
5. Paste your API key and click "Connect with API Key"

## 🔧 Advanced Configuration

### Environment Variables

All available environment variables for Google authentication:

```env
# Required for OAuth2
GOOGLE_OAUTH_CLIENT_ID=your-client-id.apps.googleusercontent.com
GOOGLE_OAUTH_CLIENT_SECRET=your-client-secret

# Optional configurations
GOOGLE_REDIRECT_PATH=/callback                    # Default callback path
GOOGLE_OAUTH_NO_BROWSER=false                    # Set to true for headless mode
GOOGLE_API_KEY=AIzaSy...                         # Alternative to OAuth2
```

### Scopes

Oxide Pilot requests the following OAuth2 scopes:

- `https://www.googleapis.com/auth/userinfo.email` - User email
- `https://www.googleapis.com/auth/userinfo.profile` - User profile
- `https://www.googleapis.com/auth/drive.file` - Google Drive file access
- `https://www.googleapis.com/auth/generative-language` - Gemini API access

### Headless Mode

For CI/CD or server environments without a browser:

```env
GOOGLE_OAUTH_NO_BROWSER=true
```

When enabled, the authentication URL will be printed to the console for manual opening.

## 🛠️ Troubleshooting

### Common Issues

1. **"OAuth client not found"**
   - Verify your Client ID is correct
   - Ensure the OAuth2 client is configured as "Desktop application"

2. **"Access blocked: This app's request is invalid"**
   - Configure the OAuth consent screen
   - Add your email to test users
   - Ensure all required fields are filled

3. **"Failed to open browser"**
   - Set `GOOGLE_OAUTH_NO_BROWSER=true` and open the URL manually
   - Check if your default browser is properly configured

4. **"Token expired"**
   - Oxide Pilot automatically refreshes tokens
   - If issues persist, clear authentication and re-authenticate

### Debug Mode

To enable detailed logging:

```bash
# Windows
$env:RUST_LOG="debug"
powershell -ExecutionPolicy Bypass -File scripts/oxide-dev.ps1

# Linux/Mac
RUST_LOG=debug ./scripts/oxide-dev.sh
```

### Clear Authentication

To reset authentication state:

1. In Oxide Pilot, go to Settings
2. Click "Clear Authentication"
3. Or manually delete stored credentials from your system keyring

## 📚 Additional Resources

- [Google Cloud Console](https://console.cloud.google.com/)
- [Google AI Studio](https://aistudio.google.com/)
- [OAuth2 Documentation](https://developers.google.com/identity/protocols/oauth2)
- [Gemini API Documentation](https://ai.google.dev/docs)

## 🔐 Security Notes

- OAuth2 tokens are stored securely in your system keyring
- Client secrets should be kept confidential
- For production deployments, use proper secret management
- Regularly rotate API keys and OAuth2 credentials
