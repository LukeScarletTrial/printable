<#
.SYNOPSIS
    Printable Installer
    Downloads the latest printable.exe compiler/build tool and sets it to PATH.
#>

[CmdletBinding()]
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Printable\bin",
    [string]$DownloadUrl = "https://raw.githubusercontent.com/LukeScarletTrial/printable/main/printable.exe"
)

$ErrorActionPreference = "Stop"

Write-Host "======================================" -ForegroundColor Cyan
Write-Host "       Printable CLI Installer        " -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan

# 1. Ensure target directory exists
if (-not (Test-Path -Path $InstallDir)) {
    Write-Host "[1/3] Creating directory: $InstallDir" -ForegroundColor Yellow
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
} else {
    Write-Host "[1/3] Directory exists: $InstallDir" -ForegroundColor Green
}

$ExePath = Join-Path $InstallDir "printable.exe"

# 2. Download printable.exe
Write-Host "[2/3] Downloading printable.exe from repository..." -ForegroundColor Yellow
try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $ExePath -UseBasicParsing
    Write-Host "      Downloaded printable.exe successfully." -ForegroundColor Green
} catch {
    Write-Warning "Failed downloading from remote ($DownloadUrl): $_"
    # Fallback to local copy if available
    $LocalCandidate = "C:\Users\MSI\.cargo\bin\printable.exe"
    if (Test-Path $LocalCandidate) {
        Write-Host "      Falling back to local pre-compiled binary..." -ForegroundColor Yellow
        Copy-Item -Path $LocalCandidate -Destination $ExePath -Force
        Write-Host "      Installed from local pre-compiled binary." -ForegroundColor Green
    } else {
        throw "Could not download printable.exe and no local candidate found."
    }
}

# 3. Add to User PATH if not already present
Write-Host "[3/3] Configuring Environment PATH..." -ForegroundColor Yellow
$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
$Paths = $UserPath -split ";" | Where-Object { $_ -ne "" }

if ($Paths -notcontains $InstallDir) {
    $NewUserPath = "$UserPath;$InstallDir"
    [Environment]::SetEnvironmentVariable("Path", $NewUserPath, [EnvironmentVariableTarget]::User)
    Write-Host "      Added '$InstallDir' to User PATH." -ForegroundColor Green
} else {
    Write-Host "      '$InstallDir' is already in User PATH." -ForegroundColor Green
}

# Update current session PATH so it is immediately usable
if ($env:Path -split ";" -notcontains $InstallDir) {
    $env:Path = "$env:Path;$InstallDir"
}

Write-Host "`nInstallation Complete!" -ForegroundColor Green
Write-Host "You can now run: printable <command> [file.bl]" -ForegroundColor Cyan
Write-Host "Example: printable init" -ForegroundColor Gray
Write-Host "         printable run <file.bl>" -ForegroundColor Gray
Write-Host "         printable build <file.bl>" -ForegroundColor Gray
