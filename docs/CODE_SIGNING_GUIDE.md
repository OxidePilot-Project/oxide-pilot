# Guía de Firma de Código para Oxide Pilot

Esta guía te ayudará a configurar la firma de código para los instaladores de Oxide Pilot.

## 🎯 Opciones de Certificados

### Opción 1: Certificado Autofirmado (Desarrollo/Testing)

**Ventajas:**
- ✅ Gratis
- ✅ Inmediato
- ✅ Ideal para testing

**Desventajas:**
- ❌ Windows SmartScreen mostrará advertencias
- ❌ No válido para distribución pública
- ❌ Usuarios deben instalar manualmente el certificado

**Cuándo usar:** Solo para desarrollo interno o testing

### Opción 2: Certificado Comercial (Producción)

**Ventajas:**
- ✅ Reconocido por Windows
- ✅ No genera advertencias de SmartScreen (después de reputación)
- ✅ Profesional y confiable

**Desventajas:**
- ❌ Costo anual (~$100-500 USD/año)
- ❌ Requiere validación de identidad

**Cuándo usar:** Para distribución pública y producción

---

## 🔧 Opción 1: Crear Certificado Autofirmado

### Paso 1: Generar Certificado Autofirmado

Abre PowerShell como **Administrador** y ejecuta:

```powershell
# Navega al directorio del proyecto
cd C:\Users\belal\Documents\oxide-pilot

# Crea directorio para certificados (ignorado por git)
New-Item -ItemType Directory -Force -Path "certs"

# Genera certificado autofirmado
$cert = New-SelfSignedCertificate `
    -Subject "CN=Oxide Pilot Development, O=Your Organization, C=US" `
    -Type CodeSigningCert `
    -KeyUsage DigitalSignature `
    -FriendlyName "Oxide Pilot Code Signing" `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -NotAfter (Get-Date).AddYears(3)

# Muestra la huella del certificado
Write-Host "✅ Certificado creado con éxito!" -ForegroundColor Green
Write-Host "Thumbprint: $($cert.Thumbprint)" -ForegroundColor Cyan
Write-Host "Subject: $($cert.Subject)" -ForegroundColor Cyan
```

### Paso 2: Exportar Certificado a PFX

```powershell
# Crear contraseña segura para el PFX
$password = Read-Host "Ingresa una contraseña segura para el certificado" -AsSecureString

# Exportar el certificado a archivo PFX
$certPath = "certs\OxidePilot-CodeSigning.pfx"
Export-PfxCertificate -Cert $cert -FilePath $certPath -Password $password

Write-Host "✅ Certificado exportado a: $certPath" -ForegroundColor Green
```

### Paso 3: Codificar PFX a Base64 (para GitHub Secrets)

```powershell
# Leer el archivo PFX y convertir a Base64
$pfxBytes = [System.IO.File]::ReadAllBytes("$PWD\certs\OxidePilot-CodeSigning.pfx")
$base64 = [System.Convert]::ToBase64String($pfxBytes)

# Guardar en archivo temporal
$base64 | Out-File -FilePath "certs\certificate-base64.txt" -Encoding ASCII

Write-Host "✅ Certificado codificado en Base64 guardado en: certs\certificate-base64.txt" -ForegroundColor Green
Write-Host "⚠️  IMPORTANTE: No compartas este archivo ni lo subas a GitHub" -ForegroundColor Yellow
```

### Paso 4: Configurar GitHub Secrets

1. Ve a tu repositorio en GitHub
2. Click en **Settings** > **Secrets and variables** > **Actions**
3. Click en **New repository secret**
4. Agrega los siguientes secretos:

#### Secret 1: SIGN_PFX_BASE64
```
Name: SIGN_PFX_BASE64
Value: [Copia el contenido completo de certs/certificate-base64.txt]
```

#### Secret 2: SIGN_PFX_PASSWORD
```
Name: SIGN_PFX_PASSWORD
Value: [La contraseña que usaste en el Paso 2]
```

#### Secret 3: SIGN_TS_URL (Opcional)
```
Name: SIGN_TS_URL
Value: http://timestamp.digicert.com
```

### Paso 5: Verificar Configuración

Ejecuta este script para verificar que todo está correcto:

```powershell
# Verificar que el certificado existe
$certPath = "certs\OxidePilot-CodeSigning.pfx"
if (Test-Path $certPath) {
    Write-Host "✅ Certificado PFX encontrado" -ForegroundColor Green
    
    # Obtener información del certificado
    $password = Read-Host "Ingresa la contraseña del certificado" -AsSecureString
    $cert = Get-PfxCertificate -FilePath $certPath -Password $password
    
    Write-Host "Subject: $($cert.Subject)" -ForegroundColor Cyan
    Write-Host "Thumbprint: $($cert.Thumbprint)" -ForegroundColor Cyan
    Write-Host "NotAfter: $($cert.NotAfter)" -ForegroundColor Cyan
    
    # Verificar que es para firma de código
    if ($cert.EnhancedKeyUsageList.FriendlyName -contains "Code Signing") {
        Write-Host "✅ Certificado válido para firma de código" -ForegroundColor Green
    } else {
        Write-Host "❌ Certificado NO es válido para firma de código" -ForegroundColor Red
    }
} else {
    Write-Host "❌ Certificado no encontrado en: $certPath" -ForegroundColor Red
}
```

---

## 🏢 Opción 2: Obtener Certificado Comercial

### Proveedores Recomendados

1. **DigiCert** (Premium, ~$400/año)
   - URL: https://www.digicert.com/signing/code-signing-certificates
   - ✅ Más confiable y reconocido
   - ✅ Soporte excelente
   - ❌ Más caro

2. **Sectigo (Comodo)** (~$200/año)
   - URL: https://sectigo.com/ssl-certificates-tls/code-signing
   - ✅ Buena relación calidad-precio
   - ✅ Ampliamente reconocido

3. **SSL.com** (~$200/año)
   - URL: https://www.ssl.com/certificates/code-signing/
   - ✅ Precios competitivos
   - ✅ Proceso rápido

### Proceso General

1. **Solicitar Certificado**
   - Visita el sitio del proveedor
   - Selecciona "Code Signing Certificate"
   - Elige Windows/Authenticode

2. **Validación de Identidad**
   - Para individuos: ID oficial + comprobante de domicilio
   - Para organizaciones: Documentos legales de la empresa
   - Puede tomar 1-5 días hábiles

3. **Recibir Certificado**
   - Recibirás un archivo .pfx o .p12
   - Guárdalo en un lugar seguro

4. **Configurar en GitHub**
   - Sigue los Pasos 3-4 de la Opción 1
   - Usa el .pfx proporcionado por el proveedor

---

## 🧪 Probar Firma Localmente

Antes de push a GitHub, puedes probar la firma localmente:

```powershell
# Construir el proyecto
cd src-tauri
cargo tauri build

# Ubicar el instalador
$installer = Get-ChildItem -Path "target\release\bundle\nsis" -Filter "*.exe" | Select-Object -First 1

if ($installer) {
    Write-Host "Instalador encontrado: $($installer.FullName)" -ForegroundColor Green
    
    # Firmar el instalador
    $certPath = "..\certs\OxidePilot-CodeSigning.pfx"
    $password = Read-Host "Contraseña del certificado" -AsSecureString
    
    # Convertir SecureString a texto plano (solo para signtool)
    $passwordText = [Runtime.InteropServices.Marshal]::PtrToStringAuto(
        [Runtime.InteropServices.Marshal]::SecureStringToBSTR($password)
    )
    
    # Firmar con signtool
    & "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe" sign `
        /f $certPath `
        /p $passwordText `
        /tr "http://timestamp.digicert.com" `
        /td sha256 `
        /fd sha256 `
        $installer.FullName
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "✅ Instalador firmado correctamente" -ForegroundColor Green
        
        # Verificar la firma
        & "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe" verify /pa $installer.FullName
    } else {
        Write-Host "❌ Error al firmar el instalador" -ForegroundColor Red
    }
} else {
    Write-Host "❌ No se encontró instalador .exe" -ForegroundColor Red
}
```

---

## 🔍 Verificar Firma de un Instalador

Para verificar que un instalador está firmado correctamente:

### Método 1: Windows Explorer

1. Click derecho en el archivo `.exe` o `.msi`
2. Selecciona **Propiedades**
3. Ve a la pestaña **Firmas digitales**
4. Deberías ver tu certificado listado
5. Click en **Detalles** para ver información completa

### Método 2: PowerShell

```powershell
# Verificar firma digital
$file = "ruta\al\instalador.exe"
$signature = Get-AuthenticodeSignature -FilePath $file

Write-Host "Estado: $($signature.Status)" -ForegroundColor $(if ($signature.Status -eq 'Valid') {'Green'} else {'Red'})
Write-Host "Firmante: $($signature.SignerCertificate.Subject)" -ForegroundColor Cyan
Write-Host "Timestamp: $($signature.TimeStamperCertificate.Subject)" -ForegroundColor Cyan
```

### Método 3: signtool

```powershell
# Verificar con signtool de Windows SDK
& "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe" verify /pa "ruta\al\instalador.exe"
```

---

## 📚 Instalación de Windows SDK (si es necesario)

Si no tienes `signtool.exe`, instala Windows SDK:

```powershell
# Descargar Windows SDK
# URL: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/

# O instalar solo signtool con Chocolatey
choco install windows-sdk-10.1 -y
```

Ubicaciones comunes de signtool:
- `C:\Program Files (x86)\Windows Kits\10\bin\<version>\x64\signtool.exe`
- `C:\Program Files (x86)\Microsoft SDKs\Windows\v10.0A\bin\NETFX 4.8 Tools\signtool.exe`

---

## 🔒 Mejores Prácticas de Seguridad

### ✅ DO (Hacer)

1. **Proteger el archivo PFX**
   - Usa contraseñas fuertes (16+ caracteres)
   - Almacena en ubicación segura cifrada
   - Nunca lo subas a GitHub

2. **Rotar certificados**
   - Renueva antes de expiración
   - Actualiza GitHub Secrets

3. **Usar Timestamp Server**
   - Asegura que las firmas permanezcan válidas después de expiración
   - Siempre incluye `/tr` en signtool

4. **Monitorear uso**
   - Revisa logs de GitHub Actions
   - Verifica que solo workflows autorizados usen el certificado

### ❌ DON'T (No hacer)

1. **No compartir contraseñas**
   - No las envíes por email/chat
   - No las guardes en texto plano

2. **No reutilizar certificados**
   - Usa certificados diferentes para dev y producción

3. **No ignorar advertencias**
   - Si un certificado falla validación, investiga

4. **No olvidar backup**
   - Mantén copias seguras del PFX
   - Documenta contraseñas en gestor seguro

---

## 🐛 Troubleshooting

### Error: "The specified PFX password is not correct"

**Solución:**
- Verifica que la contraseña en GitHub Secrets sea correcta
- Asegúrate de no tener espacios extra al copiar/pegar

### Error: "SignTool Error: No certificates were found that met all the given criteria"

**Solución:**
- El certificado no está bien codificado en Base64
- Regenera el Base64 sin saltos de línea:
  ```powershell
  $base64 = [Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx"))
  $base64 | Out-File -NoNewline cert-base64.txt
  ```

### Error: "The digital signature of the object did not verify"

**Solución:**
- El certificado expiró
- El timestamp server falló (usa uno alternativo)
- El archivo fue modificado después de firmarse

### Warning en Windows SmartScreen (Certificado Autofirmado)

**Solución:**
- Esto es normal con certificados autofirmados
- Para eliminar: usa certificado comercial
- Alternativa temporal: los usuarios pueden instalar tu certificado raíz

---

## 📊 Comparación: Autofirmado vs Comercial

| Aspecto | Autofirmado | Comercial |
|---------|-------------|-----------|
| Costo | Gratis | $100-500/año |
| Tiempo setup | 5 minutos | 1-5 días |
| SmartScreen | ⚠️ Advertencias | ✅ Sin advertencias* |
| Confianza | ❌ Baja | ✅ Alta |
| Validación | ❌ Ninguna | ✅ Identidad verificada |
| Distribución pública | ❌ No recomendado | ✅ Recomendado |
| Testing/Dev | ✅ Perfecto | ❌ Innecesario |

\* Nota: Incluso con certificado comercial, SmartScreen puede mostrar advertencias si el software es nuevo. Se requiere "reputación" (descargas verificadas) que se construye con el tiempo.

---

## 🚀 Resumen: Pasos Rápidos

### Para Desarrollo (5 minutos)

```powershell
# 1. Crear certificado autofirmado
New-SelfSignedCertificate -Subject "CN=Oxide Pilot" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"

# 2. Exportar a PFX
Export-PfxCertificate -Cert (Get-ChildItem Cert:\CurrentUser\My\<thumbprint>) -FilePath "cert.pfx" -Password (ConvertTo-SecureString "MiPassword123!" -AsPlainText -Force)

# 3. Convertir a Base64
[Convert]::ToBase64String([IO.File]::ReadAllBytes("cert.pfx")) | Out-File cert-base64.txt

# 4. Agregar a GitHub Secrets
# SIGN_PFX_BASE64 = contenido de cert-base64.txt
# SIGN_PFX_PASSWORD = MiPassword123!
```

### Para Producción (1-5 días)

1. Comprar certificado en DigiCert/Sectigo/SSL.com
2. Completar validación de identidad
3. Recibir PFX del proveedor
4. Convertir a Base64 (paso 3 arriba)
5. Agregar a GitHub Secrets

---

## 📞 Soporte

Si tienes problemas con la firma de código:

1. Revisa los logs de GitHub Actions
2. Verifica que los secretos estén configurados correctamente
3. Prueba la firma localmente primero
4. Consulta la documentación oficial de Windows: https://docs.microsoft.com/en-us/windows/win32/seccrypto/cryptography-tools

---

**¿Listo para empezar?** Sigue la guía paso a paso según tu necesidad (Desarrollo o Producción).
