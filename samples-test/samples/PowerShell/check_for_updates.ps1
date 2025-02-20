# Define the current version of your program
$currentVersion = "1.0.0"

# Define the URL where version information is stored (replace with your actual URL)
$versionCheckUrl = "https://example.com/version.txt"

try {
    # Get the latest version from remote source
    $latestVersion = Invoke-WebRequest -Uri $versionCheckUrl -UseBasicParsing | Select-Object -ExpandProperty Content

    # Compare versions
    if ($latestVersion -gt $currentVersion) {
        Write-Host "Update available! Current version: $currentVersion, Latest version: $latestVersion"
        Write-Host "Please download the latest version from our website."
    }
    else {
        Write-Host "Your program is up to date. Version: $currentVersion"
    }
}
catch {
    Write-Host "Unable to check for updates. Please check your internet connection."
    Write-Host "Error: $($_.Exception.Message)"
}
