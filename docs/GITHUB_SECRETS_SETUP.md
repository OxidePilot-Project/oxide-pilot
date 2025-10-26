# Guía: Configuración de GitHub Secrets para Oxide Pilot

## Paso 1: Generar Personal Access Token de GitHub

1. Ve a https://github.com/settings/tokens
2. Click en **"Generate new token"** > **"Generate new token (classic)"**
3. Dale un nombre descriptivo: `Oxide Pilot Secrets Config`
4. Selecciona los siguientes permisos:
   - ✅ `repo` (Full control of private repositories)
   - ✅ `workflow` (Update GitHub Action workflows)
5. Click en **"Generate token"**
6. **¡IMPORTANTE!** Copia el token inmediatamente (solo se muestra una vez)

## Paso 2: Configurar el Token en PowerShell

Abre PowerShell y ejecuta:

```powershell
$env:GITHUB_TOKEN = "tu_token_aqui"
```

## Paso 3: Instalar PyNaCl (si no está instalado)

```powershell
pip install PyNaCl
```

## Paso 4: Ejecutar el Script de Configuración

```powershell
cd e:\scripts-python\oxide-pilot
.\scripts\setup-github-secrets.ps1
```

El script automáticamente:
- ✅ Lee el certificado base64 de `certs/certificate-base64.txt`
- ✅ Lee la contraseña de `certs/password.txt`
- ✅ Encripta los secretos usando la clave pública del repositorio
- ✅ Sube los secretos a GitHub

## Secretos que se Configuran

1. **SIGN_PFX_BASE64** - Certificado de firma de código en formato Base64
2. **SIGN_PFX_PASSWORD** - Contraseña del certificado
3. **SIGN_TS_URL** - URL del servidor de timestamp (DigiCert)

## Paso 5: Verificar la Configuración

1. Ve a https://github.com/iberi22/oxide-pilot/settings/secrets/actions
2. Deberías ver los 3 secretos configurados:
   - ✅ SIGN_PFX_BASE64
   - ✅ SIGN_PFX_PASSWORD
   - ✅ SIGN_TS_URL

## Paso 6: Limpiar Archivos Sensibles

Después de configurar los secretos exitosamente:

```powershell
Remove-Item .\certs\password.txt
```

**NUNCA** subas estos archivos a GitHub:
- ❌ `certs/password.txt`
- ❌ `certs/OxidePilot-CodeSigning.pfx`
- ❌ `certs/certificate-base64.txt`

Estos archivos ya están en `.gitignore`.

## Solución de Problemas

### Error: "No se pudo obtener la clave pública"
- Verifica que el token tenga permisos `repo` y `workflow`
- Verifica que el repositorio exista: https://github.com/iberi22/oxide-pilot

### Error: "PyNaCl no está instalado"
```powershell
pip install PyNaCl
```

### Configuración Manual (si el script falla)
Si el script no funciona, configura manualmente:

1. Ve a https://github.com/iberi22/oxide-pilot/settings/secrets/actions
2. Click en **"New repository secret"**
3. Configura cada secreto:

**Secret 1:**
- Name: `SIGN_PFX_BASE64`
- Value: Copia TODO el contenido de `certs/certificate-base64.txt`

**Secret 2:**
- Name: `SIGN_PFX_PASSWORD`
- Value: Copia el contenido de `certs/password.txt`

**Secret 3:**
- Name: `SIGN_TS_URL`
- Value: `http://timestamp.digicert.com`

## Siguientes Pasos

Una vez configurados los secretos, puedes:

1. **Crear releases firmadas automáticamente** con GitHub Actions
2. **Distribuir el instalador** con firma de código válida
3. **Compilar localmente** usando los mismos certificados

Para más información, consulta:
- `docs/CODE_SIGNING_GUIDE.md`
- `docs/RELEASE_PROCESS.md`
