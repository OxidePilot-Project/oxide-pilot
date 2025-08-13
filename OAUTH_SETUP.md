# 🔐 OAuth Setup Guide for Oxide Pilot

## Overview

Oxide Pilot supports two authentication methods for Google Gemini AI:

1. **🔑 API Key** (Simple, recommended for development)
2. **🔐 OAuth 2.0** (Secure, recommended for production)

## Method 1: API Key Setup (Recommended for Testing)

### Step 1: Get Your API Key
1. Visit [Google AI Studio](https://aistudio.google.com/apikey)
2. Sign in with your Google account
3. Click "Create API Key"
4. Copy the generated API key

### Step 2: Set Environment Variable
```bash
# Windows (PowerShell)
$env:GEMINI_API_KEY="your-api-key-here"

# Windows (Command Prompt)
set GEMINI_API_KEY=your-api-key-here

# Linux/macOS
export GEMINI_API_KEY="your-api-key-here"
```

### Step 3: Run Oxide Pilot
The application will automatically detect and use the API key.

---

## Method 2: OAuth 2.0 Setup (Production Ready)

### Step 1: Create Google Cloud Project
1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project or select existing one
3. Enable the "Generative Language API"

### Step 2: Configure OAuth Consent Screen
1. Go to "APIs & Services" > "OAuth consent screen"
2. Choose "External" user type
3. Fill in required information:
   - App name: "Oxide Pilot"
   - User support email: your email
   - Developer contact: your email
4. Add scopes:
   - `https://www.googleapis.com/auth/generative-language`
   - `openid`
   - `email`
   - `profile`

### Step 3: Create OAuth Credentials
1. Go to "APIs & Services" > "Credentials"
2. Click "Create Credentials" > "OAuth 2.0 Client IDs"
3. Choose "Desktop application"
4. Name: "Oxide Pilot Desktop"
5. Add authorized redirect URIs:
   - `http://localhost:8080/callback`
   - `http://127.0.0.1:8080/callback`

### Step 4: Download Credentials
1. Download the JSON file with your credentials
2. Note the `client_id` and `client_secret`

### Step 5: Set Environment Variables
```bash
# Windows (PowerShell)
$env:GOOGLE_OAUTH_CLIENT_ID="your-client-id.apps.googleusercontent.com"
$env:GOOGLE_OAUTH_CLIENT_SECRET="your-client-secret"
$env:GOOGLE_OAUTH_REDIRECT_URI="http://localhost:8080/callback"

# Windows (Command Prompt)
set GOOGLE_OAUTH_CLIENT_ID=your-client-id.apps.googleusercontent.com
set GOOGLE_OAUTH_CLIENT_SECRET=your-client-secret
set GOOGLE_OAUTH_REDIRECT_URI=http://localhost:8080/callback

# Linux/macOS
export GOOGLE_OAUTH_CLIENT_ID="your-client-id.apps.googleusercontent.com"
export GOOGLE_OAUTH_CLIENT_SECRET="your-client-secret"
export GOOGLE_OAUTH_REDIRECT_URI="http://localhost:8080/callback"
```

### Step 6: Run Oxide Pilot
The application will use OAuth authentication and open a browser for login.

---

## Testing Authentication

### Test API Key Authentication
```bash
cargo run --bin test_auth
```

### Test OAuth Authentication
```bash
# First, remove API key to force OAuth
unset GEMINI_API_KEY  # Linux/macOS
# or
Remove-Item Env:GEMINI_API_KEY  # Windows PowerShell

# Then run OAuth test
cargo run --bin test_oauth
```

---

## Authentication Flow

### API Key Flow
1. Check for `GEMINI_API_KEY` environment variable
2. If found, use directly for API calls
3. If not found, prompt user for manual entry

### OAuth Flow
1. Check for saved OAuth token
2. If valid token exists, use it
3. If token expired, refresh using refresh token
4. If no token or refresh fails:
   - Open browser to Google OAuth
   - User grants permissions
   - Receive authorization code
   - Exchange code for access token
   - Save token for future use

---

## Security Considerations

### API Key
- ✅ Simple to set up
- ✅ Good for development/testing
- ⚠️ No automatic expiration
- ⚠️ Manual key management required

### OAuth
- ✅ Automatic token refresh
- ✅ User controls permissions
- ✅ Tokens expire automatically
- ✅ More secure for production
- ⚠️ More complex setup

---

## Troubleshooting

### Common Issues

#### "Authentication failed: OAuth error"
- Check that OAuth credentials are correctly set
- Verify redirect URI matches exactly
- Ensure Generative Language API is enabled

#### "Invalid API key"
- Verify API key is correct
- Check that API key has proper permissions
- Ensure Generative Language API is enabled for your project

#### "Browser doesn't open"
- Check firewall settings
- Try manually opening the URL shown in console
- Ensure port 8080 is available

#### "Token refresh failed"
- Clear saved authentication: `cargo run --bin test_auth` and choose clear option
- Re-authenticate with fresh OAuth flow

---

## Production Deployment

For production deployment:

1. **Use OAuth** instead of API keys
2. **Secure credential storage** (not environment variables)
3. **Implement proper error handling**
4. **Monitor token usage and refresh**
5. **Consider service account authentication** for server deployments

---

## Next Steps

Once authentication is working:

1. **Test AI conversations**: Use the chat interface
2. **Enable voice features**: Configure microphone permissions
3. **Set up monitoring**: Configure security alerts
4. **Customize settings**: Adjust AI behavior and security policies

For support, check the main README.md or create an issue on GitHub.
