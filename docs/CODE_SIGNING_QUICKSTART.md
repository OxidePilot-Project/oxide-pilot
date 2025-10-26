# 🔐 Guía Rápida: Configuración de Firma de Código

Esta guía te ayudará a configurar la firma de código para Oxide Pilot en **5 minutos**.

## 🚀 Inicio Rápido (Desarrollo)

### Paso 1: Ejecutar el Script de Setup

Abre PowerShell **como Administrador** y ejecuta:

```powershell
cd C:\Users\belal\Documents\oxide-pilot
.\scripts\setup-code-signing.ps1
```

El script te guiará paso a paso:
1. ✅ Crea certificado autofirmado
2. ✅ Genera contraseña segura (o permite ingresar una propia)
3. ✅ Exporta certificado a PFX
4. ✅ Codifica a Base64 para GitHub
5. ✅ Verifica que todo funciona

### Paso 2: Configurar GitHub Secrets

1. Ve a: https://github.com/YOUR_USERNAME/oxide-pilot/settings/secrets/actions

2. Click en **"New repository secret"**

3. Agrega el primer secret:
   ```
   Name: SIGN_PFX_BASE64
   Value: [Copia TODO el contenido de certs/certificate-base64.txt]
   ```

4. Agrega el segundo secret:
   ```
   Name: SIGN_PFX_PASSWORD
   Value: [Copia el contenido de certs/password.txt o tu contraseña]
   ```

5. (Opcional) Agrega timestamp server:
   ```
   Name: SIGN_TS_URL
   Value: http://timestamp.digicert.com
   ```

### Paso 3: ¡Listo!

Ahora cuando hagas push a `main` o crees un tag, los instaladores se firmarán automáticamente.

---

## 🧪 Probar Firma Localmente

Antes de push, puedes probar que la firma funciona:

```powershell
# 1. Construir el proyecto
cd src-tauri
cargo tauri build

# 2. El instalador firmado estará en:
# target/release/bundle/nsis/*.exe
# target/release/bundle/msi/*.msi

# 3. Verificar la firma
Get-AuthenticodeSignature target/release/bundle/nsis/*.exe
```

---

## 📊 Verificar que GitHub Actions Funciona

Después de configurar los secrets:

1. Haz un pequeño cambio (ej: editar README.md)
2. Commit y push a main
3. Ve a: https://github.com/YOUR_USERNAME/oxide-pilot/actions
4. Observa el workflow "Automated Release"
5. Si todo está bien, verás: ✅ Code sign installers (optional)

---

## ⚠️ Seguridad

### ✅ Hacer:
- Guardar la contraseña en un gestor de passwords
- Hacer backup del archivo PFX en un lugar seguro (cifrado)
- Eliminar `certs/password.txt` después de configurar GitHub

### ❌ NO Hacer:
- **NUNCA** subir archivos de `certs/` a GitHub
- **NUNCA** compartir el PFX o la contraseña
- **NUNCA** commitear `*.pfx` o `*password*.txt`

---

## 🏢 Para Producción (Certificado Comercial)

Si quieres un certificado comercial reconocido:

1. **Comprar certificado** en:
   - DigiCert (~$400/año): https://www.digicert.com/
   - Sectigo (~$200/año): https://sectigo.com/
   - SSL.com (~$200/año): https://www.ssl.com/

2. **Completar validación** (1-5 días hábiles)

3. **Recibir PFX** del proveedor

4. **Configurar en GitHub**:
   ```powershell
   # Convertir PFX a Base64
   $pfxBytes = [IO.File]::ReadAllBytes("tu-certificado-comercial.pfx")
   $base64 = [Convert]::ToBase64String($pfxBytes)
   $base64 | Out-File -NoNewline certificate-base64.txt
   
   # Usar este Base64 en SIGN_PFX_BASE64
   ```

---

## 🐛 Problemas Comunes

### "The specified PFX password is not correct"

**Solución:** Verifica que la contraseña en GitHub Secrets sea exacta (sin espacios extra).

### "SignTool not found"

**Solución:** Instala Windows SDK:
```powershell
choco install windows-sdk-10.1 -y
```

### SmartScreen muestra advertencia

**Esto es normal** con certificados autofirmados. Para eliminarlo:
- Usa certificado comercial
- O: los usuarios deben instalar tu certificado raíz manualmente

---

## 📚 Más Información

Para detalles completos, consulta:
- **docs/CODE_SIGNING_GUIDE.md** - Guía completa paso a paso
- **.github/RELEASE_AUTOMATION.md** - Documentación del sistema de releases

---

## 🎯 Resumen

1. ✅ Ejecuta `.\scripts\setup-code-signing.ps1` como Admin
2. ✅ Copia Base64 y contraseña a GitHub Secrets
3. ✅ Push a main o crea tag
4. ✅ ¡Los instaladores se firman automáticamente!

**Tiempo total: ~5 minutos** ⏱️
