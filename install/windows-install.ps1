$ErrorActionPreference = 'Stop'
$Repository = 'Dank-hunters/Branlly_next'
$Headers = @{ 'User-Agent' = 'Branlly-Installer' }

if (-not [Environment]::Is64BitOperatingSystem) {
    throw 'Branlly Next nécessite Windows 64 bits.'
}

$release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repository/releases/latest" -Headers $Headers
$installer = $release.assets |
    Where-Object { $_.name -match '(?i)(x64|amd64).*\.msi$|\.msi$' } |
    Select-Object -First 1
if (-not $installer) {
    $installer = $release.assets |
        Where-Object { $_.name -match '(?i)(x64|amd64).*\.exe$|setup.*\.exe$' } |
        Select-Object -First 1
}
if (-not $installer) { throw 'Aucun installateur Windows trouvé dans la dernière release.' }

$checksumAsset = $release.assets |
    Where-Object { $_.name -eq "$($installer.name).sha256" } |
    Select-Object -First 1
if (-not $checksumAsset) { throw 'Le fichier de contrôle SHA-256 est absent.' }

$temporary = Join-Path ([IO.Path]::GetTempPath()) "branlly-$([guid]::NewGuid())"
New-Item -ItemType Directory -Path $temporary | Out-Null
try {
    $installerPath = Join-Path $temporary $installer.name
    $checksumPath = "$installerPath.sha256"
    Invoke-WebRequest -Uri $installer.browser_download_url -Headers $Headers -OutFile $installerPath
    Invoke-WebRequest -Uri $checksumAsset.browser_download_url -Headers $Headers -OutFile $checksumPath

    $expected = ((Get-Content $checksumPath -Raw).Trim() -split '\s+')[0].ToLowerInvariant()
    $actual = (Get-FileHash $installerPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { throw 'Échec de la vérification SHA-256 de l’installateur.' }

    if ([IO.Path]::GetExtension($installerPath) -eq '.msi') {
        Start-Process msiexec.exe -ArgumentList @('/i', "`"$installerPath`"") -Wait
    } else {
        Start-Process $installerPath -Wait
    }
} finally {
    Remove-Item $temporary -Recurse -Force -ErrorAction SilentlyContinue
}
