# Oxide Pilot - Security Diagnostic Tool
# Análisis de seguridad del sistema para detectar posibles intrusiones

param(
    [switch]$Verbose,
    [switch]$SaveReport,
    [string]$OutputPath = "security-report.json"
)

Write-Host "=== OXIDE PILOT - DIAGNÓSTICO DE SEGURIDAD ===" -ForegroundColor Cyan
Write-Host "Analizando el sistema en busca de posibles amenazas..." -ForegroundColor Yellow
Write-Host ""

$results = @{
    timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    suspicious_findings = @()
    network_analysis = @()
    process_analysis = @()
    file_integrity = @()
    recommendations = @()
}

# 1. ANÁLISIS DE PROCESOS SOSPECHOSOS
Write-Host "[1/6] Analizando procesos en ejecución..." -ForegroundColor Green

$suspiciousProcesses = @()
$allProcesses = Get-Process | Where-Object { $_.ProcessName -ne $null }

foreach ($proc in $allProcesses) {
    $suspicious = $false
    $reasons = @()

    try {
        # Procesos sin ruta (posible malware en memoria)
        if (-not $proc.Path) {
            $suspicious = $true
            $reasons += "Sin ruta de archivo ejecutable"
        }

        # Procesos con nombres sospechosos
        $suspiciousNames = @(
            "keylogger", "ratserver", "backdoor", "trojan", "rootkit",
            "mimikatz", "psexec", "metasploit", "netcat", "ncat"
        )

        foreach ($name in $suspiciousNames) {
            if ($proc.ProcessName -like "*$name*") {
                $suspicious = $true
                $reasons += "Nombre sospechoso: contiene '$name'"
            }
        }

        # Procesos con uso elevado de CPU sin interfaz gráfica
        if ($proc.CPU -gt 50 -and -not $proc.MainWindowTitle) {
            $suspicious = $true
            $reasons += "Alto uso de CPU sin ventana visible"
        }

        if ($suspicious) {
            $suspiciousProcesses += [PSCustomObject]@{
                Name = $proc.ProcessName
                PID = $proc.Id
                Path = $proc.Path
                CPU = $proc.CPU
                Memory_MB = [math]::Round($proc.WorkingSet64 / 1MB, 2)
                StartTime = if ($proc.StartTime) { $proc.StartTime.ToString() } else { "Desconocido" }
                Reasons = ($reasons -join ", ")
            }
        }
    } catch {
        # Ignorar errores de acceso
    }
}

if ($suspiciousProcesses.Count -gt 0) {
    Write-Host "  ⚠️  ENCONTRADOS $($suspiciousProcesses.Count) PROCESOS SOSPECHOSOS" -ForegroundColor Red
    $results.suspicious_findings += "Procesos sospechosos detectados: $($suspiciousProcesses.Count)"
    $results.process_analysis = $suspiciousProcesses

    if ($Verbose) {
        $suspiciousProcesses | Format-Table -AutoSize
    }
} else {
    Write-Host "  ✓ No se detectaron procesos sospechosos" -ForegroundColor Green
}

# 2. ANÁLISIS DE CONEXIONES DE RED
Write-Host "[2/6] Analizando conexiones de red activas..." -ForegroundColor Green

$suspiciousConnections = @()
$connections = Get-NetTCPConnection -State Established -ErrorAction SilentlyContinue

foreach ($conn in $connections) {
    $suspicious = $false
    $reasons = @()

    # Puertos comúnmente usados por malware
    $suspiciousPorts = @(4444, 5555, 6666, 7777, 8888, 31337, 12345, 54321)

    if ($suspiciousPorts -contains $conn.RemotePort) {
        $suspicious = $true
        $reasons += "Puerto sospechoso: $($conn.RemotePort)"
    }

    # Conexiones a IPs privadas sospechosas desde procesos no del sistema
    if ($conn.RemoteAddress -match "^(192\.168\.|10\.|172\.(1[6-9]|2[0-9]|3[01])\.)" -and
        $conn.OwningProcess -gt 1000) {
        try {
            $proc = Get-Process -Id $conn.OwningProcess -ErrorAction Stop
            if ($proc.Path -notmatch "Windows|Program Files") {
                $suspicious = $true
                $reasons += "Conexión interna desde proceso no del sistema"
            }
        } catch {}
    }

    if ($suspicious) {
        try {
            $proc = Get-Process -Id $conn.OwningProcess -ErrorAction Stop
            $suspiciousConnections += [PSCustomObject]@{
                LocalAddress = "$($conn.LocalAddress):$($conn.LocalPort)"
                RemoteAddress = "$($conn.RemoteAddress):$($conn.RemotePort)"
                State = $conn.State
                Process = $proc.ProcessName
                PID = $conn.OwningProcess
                Reasons = ($reasons -join ", ")
            }
        } catch {}
    }
}

if ($suspiciousConnections.Count -gt 0) {
    Write-Host "  ⚠️  ENCONTRADAS $($suspiciousConnections.Count) CONEXIONES SOSPECHOSAS" -ForegroundColor Red
    $results.suspicious_findings += "Conexiones de red sospechosas: $($suspiciousConnections.Count)"
    $results.network_analysis = $suspiciousConnections

    if ($Verbose) {
        $suspiciousConnections | Format-Table -AutoSize
    }
} else {
    Write-Host "  ✓ No se detectaron conexiones sospechosas" -ForegroundColor Green
}

# 3. ANÁLISIS DE ARCHIVOS DE INICIO AUTOMÁTICO
Write-Host "[3/6] Verificando programas de inicio automático..." -ForegroundColor Green

$suspiciousStartup = @()
$startupLocations = @(
    "HKLM:\Software\Microsoft\Windows\CurrentVersion\Run",
    "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run",
    "HKLM:\Software\Microsoft\Windows\CurrentVersion\RunOnce",
    "HKCU:\Software\Microsoft\Windows\CurrentVersion\RunOnce"
)

foreach ($location in $startupLocations) {
    try {
        $items = Get-ItemProperty -Path $location -ErrorAction Stop
        $items.PSObject.Properties | Where-Object { $_.Name -notlike "PS*" } | ForEach-Object {
            $path = $_.Value
            $suspicious = $false
            $reasons = @()

            # Verificar ubicaciones sospechosas
            if ($path -match "AppData\\Roaming|Temp|Public") {
                $suspicious = $true
                $reasons += "Ubicación inusual para programa de inicio"
            }

            # Verificar nombres ofuscados
            if ($path -match "^[a-z0-9]{8,}\.exe$") {
                $suspicious = $true
                $reasons += "Nombre de archivo aleatorio/ofuscado"
            }

            if ($suspicious) {
                $suspiciousStartup += [PSCustomObject]@{
                    Name = $_.Name
                    Path = $path
                    Location = $location
                    Reasons = ($reasons -join ", ")
                }
            }
        }
    } catch {}
}

if ($suspiciousStartup.Count -gt 0) {
    Write-Host "  ⚠️  ENCONTRADOS $($suspiciousStartup.Count) PROGRAMAS DE INICIO SOSPECHOSOS" -ForegroundColor Red
    $results.suspicious_findings += "Programas de inicio sospechosos: $($suspiciousStartup.Count)"
    $results.file_integrity += $suspiciousStartup

    if ($Verbose) {
        $suspiciousStartup | Format-Table -AutoSize
    }
} else {
    Write-Host "  ✓ No se detectaron programas de inicio sospechosos" -ForegroundColor Green
}

# 4. ANÁLISIS DE SERVICIOS
Write-Host "[4/6] Analizando servicios del sistema..." -ForegroundColor Green

$suspiciousServices = @()
$services = Get-Service | Where-Object { $_.Status -eq "Running" }

foreach ($svc in $services) {
    try {
        $svcDetails = Get-WmiObject Win32_Service -Filter "Name='$($svc.Name)'" -ErrorAction Stop
        $suspicious = $false
        $reasons = @()

        # Servicios sin descripción
        if (-not $svcDetails.Description -or $svcDetails.Description.Trim() -eq "") {
            $suspicious = $true
            $reasons += "Sin descripción"
        }

        # Servicios en ubicaciones no estándar
        if ($svcDetails.PathName -match "AppData|Temp|Users\\Public") {
            $suspicious = $true
            $reasons += "Ubicación no estándar"
        }

        # Servicios con nombres aleatorios
        if ($svc.Name -match "^[a-z0-9]{8,}$") {
            $suspicious = $true
            $reasons += "Nombre aleatorio"
        }

        if ($suspicious) {
            $suspiciousServices += [PSCustomObject]@{
                Name = $svc.Name
                DisplayName = $svc.DisplayName
                Path = $svcDetails.PathName
                StartMode = $svcDetails.StartMode
                Reasons = ($reasons -join ", ")
            }
        }
    } catch {}
}

if ($suspiciousServices.Count -gt 0) {
    Write-Host "  ⚠️  ENCONTRADOS $($suspiciousServices.Count) SERVICIOS SOSPECHOSOS" -ForegroundColor Red
    $results.suspicious_findings += "Servicios sospechosos: $($suspiciousServices.Count)"

    if ($Verbose) {
        $suspiciousServices | Format-Table -AutoSize
    }
} else {
    Write-Host "  ✓ No se detectaron servicios sospechosos" -ForegroundColor Green
}

# 5. VERIFICACIÓN DE ARCHIVOS HOSTS Y DNS
Write-Host "[5/6] Verificando configuración de DNS y archivo hosts..." -ForegroundColor Green

$dnsIssues = @()

# Verificar archivo hosts
$hostsPath = "$env:SystemRoot\System32\drivers\etc\hosts"
if (Test-Path $hostsPath) {
    $hostsContent = Get-Content $hostsPath
    $modifiedLines = $hostsContent | Where-Object {
        $_ -notmatch "^#" -and $_ -notmatch "^\s*$" -and $_ -notmatch "localhost"
    }

    if ($modifiedLines.Count -gt 0) {
        Write-Host "  ⚠️  ARCHIVO HOSTS MODIFICADO" -ForegroundColor Red
        $results.suspicious_findings += "Archivo hosts contiene entradas personalizadas"
        $dnsIssues += [PSCustomObject]@{
            Type = "Hosts File"
            Issue = "Contiene $($modifiedLines.Count) líneas personalizadas"
            Details = ($modifiedLines -join "; ")
        }
    } else {
        Write-Host "  ✓ Archivo hosts sin modificaciones" -ForegroundColor Green
    }
}

# Verificar servidores DNS
$dnsServers = Get-DnsClientServerAddress -AddressFamily IPv4 |
    Where-Object { $_.ServerAddresses.Count -gt 0 }

foreach ($dns in $dnsServers) {
    # DNS públicos conocidos y confiables
    $knownGoodDNS = @("8.8.8.8", "8.8.4.4", "1.1.1.1", "1.0.0.1", "208.67.222.222", "208.67.220.220")

    foreach ($server in $dns.ServerAddresses) {
        if ($server -notmatch "^(192\.168\.|10\.|172\.(1[6-9]|2[0-9]|3[01])\.|127\.)" -and
            $server -notin $knownGoodDNS) {
            $dnsIssues += [PSCustomObject]@{
                Type = "DNS Server"
                Interface = $dns.InterfaceAlias
                Issue = "Servidor DNS no estándar: $server"
                Details = "Podría ser un DNS malicioso"
            }
        }
    }
}

if ($dnsIssues.Count -gt 0) {
    Write-Host "  ⚠️  PROBLEMAS DE DNS/HOSTS DETECTADOS" -ForegroundColor Red
    if ($Verbose) {
        $dnsIssues | Format-Table -AutoSize
    }
} else {
    Write-Host "  ✓ Configuración DNS correcta" -ForegroundColor Green
}

# 6. VERIFICACIÓN DE TAREAS PROGRAMADAS
Write-Host "[6/6] Analizando tareas programadas..." -ForegroundColor Green

$suspiciousTasks = @()
$tasks = Get-ScheduledTask | Where-Object { $_.State -ne "Disabled" }

foreach ($task in $tasks) {
    $suspicious = $false
    $reasons = @()

    try {
        $taskInfo = $task | Get-ScheduledTaskInfo -ErrorAction Stop
        $actions = $task.Actions

        foreach ($action in $actions) {
            if ($action.Execute) {
                # Verificar scripts de PowerShell ofuscados
                if ($action.Execute -match "powershell|pwsh" -and
                    $action.Arguments -match "-enc|-e |-w hidden") {
                    $suspicious = $true
                    $reasons += "Script PowerShell codificado u oculto"
                }

                # Verificar ubicaciones sospechosas
                if ($action.Execute -match "AppData\\Roaming|Temp|Public") {
                    $suspicious = $true
                    $reasons += "Ejecutable en ubicación sospechosa"
                }
            }
        }

        if ($suspicious) {
            $suspiciousTasks += [PSCustomObject]@{
                Name = $task.TaskName
                Path = $task.TaskPath
                State = $task.State
                LastRun = $taskInfo.LastRunTime
                NextRun = $taskInfo.NextRunTime
                Action = ($actions | ForEach-Object { "$($_.Execute) $($_.Arguments)" }) -join "; "
                Reasons = ($reasons -join ", ")
            }
        }
    } catch {}
}

if ($suspiciousTasks.Count -gt 0) {
    Write-Host "  ⚠️  ENCONTRADAS $($suspiciousTasks.Count) TAREAS PROGRAMADAS SOSPECHOSAS" -ForegroundColor Red
    $results.suspicious_findings += "Tareas programadas sospechosas: $($suspiciousTasks.Count)"

    if ($Verbose) {
        $suspiciousTasks | Format-Table -AutoSize
    }
} else {
    Write-Host "  ✓ No se detectaron tareas programadas sospechosas" -ForegroundColor Green
}

# RESUMEN Y RECOMENDACIONES
Write-Host ""
Write-Host "=== RESUMEN DEL ANÁLISIS ===" -ForegroundColor Cyan

$totalIssues = $results.suspicious_findings.Count

if ($totalIssues -eq 0) {
    Write-Host "✓ No se detectaron amenazas evidentes en el sistema" -ForegroundColor Green
    Write-Host "Tu computador parece estar limpio." -ForegroundColor Green
    $results.recommendations += "Sistema aparentemente limpio. Mantén actualizado tu antivirus y sistema operativo."
} else {
    Write-Host "⚠️  SE DETECTARON $totalIssues HALLAZGOS SOSPECHOSOS" -ForegroundColor Red
    Write-Host ""
    Write-Host "Hallazgos:" -ForegroundColor Yellow
    foreach ($finding in $results.suspicious_findings) {
        Write-Host "  - $finding" -ForegroundColor Yellow
    }

    Write-Host ""
    Write-Host "RECOMENDACIONES:" -ForegroundColor Red
    $recommendations = @(
        "1. Ejecutar un análisis completo con Windows Defender o tu antivirus",
        "2. Verificar manualmente los procesos y conexiones sospechosas listadas",
        "3. Desconectar temporalmente de internet si sospechas compromiso activo",
        "4. Cambiar contraseñas importantes desde un dispositivo seguro",
        "5. Considerar una limpieza profesional si persisten las sospechas",
        "6. Revisar logs de eventos de Windows para actividad inusual",
        "7. Verificar extensiones de navegador y programas recientemente instalados"
    )

    foreach ($rec in $recommendations) {
        Write-Host "  $rec" -ForegroundColor Yellow
        $results.recommendations += $rec
    }
}

# Guardar reporte si se solicitó
if ($SaveReport) {
    $reportData = @{
        analysis_results = $results
        suspicious_processes = $suspiciousProcesses
        suspicious_connections = $suspiciousConnections
        suspicious_startup = $suspiciousStartup
        suspicious_services = $suspiciousServices
        dns_issues = $dnsIssues
        suspicious_tasks = $suspiciousTasks
    }

    $reportData | ConvertTo-Json -Depth 10 | Out-File $OutputPath -Encoding UTF8
    Write-Host ""
    Write-Host "📄 Reporte guardado en: $OutputPath" -ForegroundColor Cyan
}

Write-Host ""
Write-Host "=== FIN DEL DIAGNÓSTICO ===" -ForegroundColor Cyan
