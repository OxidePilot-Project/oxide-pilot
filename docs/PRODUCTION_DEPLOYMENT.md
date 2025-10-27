# 🚀 Production Deployment Guide

**Version**: 1.0.0
**Last Updated**: October 27, 2025

---

## 📋 Pre-Deployment Checklist

### Code Quality

- [x] All tests passing (36/36)
- [x] Zero Clippy warnings
- [x] Code formatted with rustfmt
- [x] Documentation complete
- [x] Security audit passed

### Performance

- [x] Binary size optimized (<50MB)
- [x] Memory usage optimized (<200MB)
- [x] CPU usage optimized (<2% idle)
- [x] Startup time optimized (<3s)

### Security

- [x] OAuth2 authentication implemented
- [x] API keys encrypted
- [x] RPA permissions system active
- [x] Audit logging enabled
- [x] YARA threat detection active

---

## 🔧 Build Process

### 1. Automated Production Build

```powershell
# Run the production build script
.\scripts\build-production.ps1
```

This script will:
1. Clean previous builds
2. Run all tests
3. Execute Clippy linting
4. Build frontend (optimized)
5. Build backend (release mode)
6. Report binary size

### 2. Manual Build (Alternative)

```bash
# Frontend
cd src-frontend
npm install
npm run build

# Backend
cd ../src-tauri
cargo build --release --features surrealdb-metrics
```

---

## 📦 Build Artifacts

### Locations

```
oxide-pilot/
├── src-tauri/target/release/
│   └── oxide-pilot.exe          # Main executable
├── src-frontend/dist/            # Frontend assets
└── data/                         # Database (created at runtime)
```

### Sizes

| Artifact | Size | Notes |
|----------|------|-------|
| oxide-pilot.exe | ~45MB | Stripped, optimized |
| Frontend bundle | ~2MB | Minified, gzipped |
| Total | ~47MB | First install |

---

## 🖥️ System Requirements

### Minimum Requirements

| Component | Requirement |
|-----------|-------------|
| **OS** | Windows 10/11, macOS 11+, Linux (Ubuntu 20.04+) |
| **CPU** | 2 cores, 2.0 GHz |
| **RAM** | 4 GB |
| **Disk** | 500 MB free space |
| **Network** | Internet connection (for AI providers) |

### Recommended Requirements

| Component | Requirement |
|-----------|-------------|
| **OS** | Windows 11, macOS 13+, Linux (Ubuntu 22.04+) |
| **CPU** | 4+ cores, 3.0+ GHz |
| **RAM** | 8+ GB |
| **Disk** | 2+ GB free space (for logs and database) |
| **Network** | Broadband connection |

---

## 🔐 Security Configuration

### 1. API Keys and Secrets

**Environment Variables**:

```bash
# Google Gemini (OAuth2)
GOOGLE_CLIENT_ID=your_client_id
GOOGLE_CLIENT_SECRET=your_client_secret

# Qwen (OAuth2)
QWEN_CLIENT_ID=your_client_id
QWEN_CLIENT_SECRET=your_client_secret
QWEN_API_BASE=https://dashscope.aliyuncs.com

# OpenAI (API Key)
OPENAI_API_KEY=your_api_key

# Optional: Custom database path
OXIDE_DB_PATH=./data/oxide.db
```

**Storage**:
- API keys stored in system keyring
- OAuth tokens encrypted at rest
- Secure credential management

### 2. RPA Permissions

**Default Configuration**:

```rust
// Deny-by-default security model
PermissionLevel::Deny  // All actions require explicit approval
```

**Customization**:
- Configure in Settings > RPA
- Granular permission control
- Audit logging enabled

---

## 🚀 Deployment Methods

### Method 1: Standalone Executable

**Windows**:

```powershell
# Copy executable
Copy-Item src-tauri\target\release\oxide-pilot.exe C:\Program Files\OxidePilot\

# Create shortcut
$WshShell = New-Object -comObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$Home\Desktop\Oxide Pilot.lnk")
$Shortcut.TargetPath = "C:\Program Files\OxidePilot\oxide-pilot.exe"
$Shortcut.Save()
```

**macOS**:

```bash
# Create app bundle
cargo tauri build --target universal-apple-darwin

# Install
cp -r src-tauri/target/release/bundle/macos/Oxide\ Pilot.app /Applications/
```

**Linux**:

```bash
# Build AppImage
cargo tauri build --target x86_64-unknown-linux-gnu

# Install
sudo cp src-tauri/target/release/oxide-pilot /usr/local/bin/
```

### Method 2: Installer Package

**Windows (MSI)**:

```powershell
# Build installer
cargo tauri build --target x86_64-pc-windows-msvc

# Output: src-tauri/target/release/bundle/msi/Oxide Pilot_0.1.0_x64_en-US.msi
```

**macOS (DMG)**:

```bash
# Build DMG
cargo tauri build --target universal-apple-darwin

# Output: src-tauri/target/release/bundle/dmg/Oxide Pilot_0.1.0_universal.dmg
```

**Linux (DEB/RPM)**:

```bash
# Build DEB
cargo tauri build --target x86_64-unknown-linux-gnu

# Output: src-tauri/target/release/bundle/deb/oxide-pilot_0.1.0_amd64.deb
```

---

## 🔄 Update Strategy

### 1. Auto-Update Configuration

**tauri.conf.json**:

```json
{
  "updater": {
    "active": true,
    "endpoints": [
      "https://releases.oxidepilot.com/{{target}}/{{current_version}}"
    ],
    "dialog": true,
    "pubkey": "YOUR_PUBLIC_KEY"
  }
}
```

### 2. Manual Update

```bash
# Download latest release
# Extract and replace executable
# Restart application
```

---

## 📊 Monitoring and Logging

### 1. Application Logs

**Location**:

```
Windows: %APPDATA%\com.oxidepilot.app\logs\
macOS: ~/Library/Application Support/com.oxidepilot.app/logs/
Linux: ~/.local/share/com.oxidepilot.app/logs/
```

**Log Levels**:

```bash
# Set via environment variable
RUST_LOG=info          # Production (default)
RUST_LOG=debug         # Development
RUST_LOG=trace         # Debugging
```

### 2. Performance Monitoring

**Built-in Dashboard**:
- Guardian tab shows real-time metrics
- CPU, Memory, Disk, Network monitoring
- Alert system for anomalies

**External Monitoring** (Optional):
- Prometheus metrics export
- Grafana dashboards
- Custom alerting

---

## 🐛 Troubleshooting

### Common Issues

#### 1. Application Won't Start

**Symptoms**: Executable crashes immediately

**Solutions**:
```bash
# Check dependencies
ldd oxide-pilot  # Linux
otool -L oxide-pilot  # macOS

# Check logs
cat ~/.local/share/com.oxidepilot.app/logs/oxide-pilot.log

# Verify permissions
chmod +x oxide-pilot
```

#### 2. High Memory Usage

**Symptoms**: Memory usage > 500MB

**Solutions**:
```bash
# Check for memory leaks
valgrind --leak-check=full ./oxide-pilot

# Reduce refresh intervals
# Settings > Guardian > Monitoring Interval: 10s → 30s
```

#### 3. Database Corruption

**Symptoms**: Application crashes on startup

**Solutions**:
```bash
# Backup database
cp data/oxide.db data/oxide.db.backup

# Reset database
rm data/oxide.db
# Application will recreate on next start
```

---

## 🔒 Security Best Practices

### 1. Production Deployment

✅ **Do**:
- Use HTTPS for all API calls
- Enable audit logging
- Implement rate limiting
- Use strong authentication
- Regular security updates

❌ **Don't**:
- Expose API keys in logs
- Disable security features
- Run as administrator/root
- Use default credentials
- Skip security updates

### 2. Data Protection

**Encryption**:
- API keys encrypted at rest
- OAuth tokens encrypted
- Database encryption (optional)

**Backup**:
```bash
# Backup database
cp data/oxide.db backups/oxide-$(date +%Y%m%d).db

# Backup configuration
cp -r ~/.config/oxidepilot backups/config-$(date +%Y%m%d)
```

---

## 📈 Performance Tuning

### 1. Resource Limits

**Cargo.toml**:

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
```

### 2. Runtime Configuration

**Environment Variables**:

```bash
# Tokio worker threads
TOKIO_WORKER_THREADS=4

# Database cache size
SURREALDB_CACHE_SIZE=100MB

# Metrics collection interval
OXIDE_METRICS_INTERVAL=5
```

---

## 🔄 Rollback Procedure

### If Deployment Fails

1. **Stop Application**
   ```bash
   pkill oxide-pilot
   ```

2. **Restore Previous Version**
   ```bash
   cp backups/oxide-pilot.exe.backup oxide-pilot.exe
   ```

3. **Restore Database**
   ```bash
   cp backups/oxide.db.backup data/oxide.db
   ```

4. **Restart Application**
   ```bash
   ./oxide-pilot
   ```

---

## 📞 Support

### Getting Help

- **Documentation**: [docs/](../docs/)
- **Issues**: [GitHub Issues](https://github.com/OxidePilot-Project/oxide-pilot/issues)
- **Discussions**: [GitHub Discussions](https://github.com/OxidePilot-Project/oxide-pilot/discussions)

### Reporting Bugs

Include:
- OS and version
- Application version
- Steps to reproduce
- Error logs
- Screenshots (if applicable)

---

## ✅ Post-Deployment Checklist

- [ ] Application starts successfully
- [ ] Authentication works (Gemini/Qwen/OpenAI)
- [ ] Guardian dashboard shows metrics
- [ ] RPA system functional
- [ ] Logs are being written
- [ ] Database is being populated
- [ ] Performance is acceptable
- [ ] No security warnings
- [ ] Auto-update configured (if applicable)
- [ ] Backup strategy in place

---

**Maintained by**: Oxide Pilot Team
**Last Updated**: October 27, 2025
