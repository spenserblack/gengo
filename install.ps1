# Create gengo directory in AppData/Roaming
$gengoPath = "$env:APPDATA\gengo"
New-Item -ItemType Directory -Force -Path $gengoPath -ErrorAction SilentlyContinue | Out-Null
Write-Host "Installing gengo to $gengoPath"

# Create and use temporary directory
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Force -Path $tempDir -ErrorAction SilentlyContinue | Out-Null
Push-Location $tempDir

try {
    # Download and extract gengo
    $tempFile = "gengo-installer.tar.gz"
    Invoke-WebRequest -Uri "https://github.com/spenserblack/gengo/releases/latest/download/gengo-x86_64-pc-windows-msvc.tar.gz" -OutFile $tempFile
    tar -xzf $tempFile

    # Move executable to AppData/Roaming/gengo
    Move-Item -Force "gengo.exe" $gengoPath

    # Add to PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$gengoPath*") {
        [Environment]::SetEnvironmentVariable("Path", "$userPath;$gengoPath", "User")
    }
} finally {
    # Clean up and restore location
    Pop-Location
    Remove-Item -Recurse -Force $tempDir -ErrorAction SilentlyContinue
}

Write-Host "Gengo has been installed and added to your PATH. Please restart your terminal for the changes to take effect."
