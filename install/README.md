# Installateurs

Les scripts récupèrent uniquement la dernière GitHub Release officielle et vérifient son SHA-256 avant installation.

## Linux x86_64

```bash
curl -fsSLO https://raw.githubusercontent.com/Dank-hunters/Branlly_next/main/install/linux-install.sh
less linux-install.sh
bash linux-install.sh
```

## Windows 64 bits

Dans PowerShell :

```powershell
Invoke-WebRequest https://raw.githubusercontent.com/Dank-hunters/Branlly_next/main/install/windows-install.ps1 -OutFile windows-install.ps1
Get-Content .\windows-install.ps1
powershell -ExecutionPolicy Bypass -File .\windows-install.ps1
```

Les scripts échouent proprement si aucune release compatible ou aucun checksum n’est disponible.
