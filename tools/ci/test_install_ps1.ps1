# install.ps1 against stub toolchains in release-shaped zips, served through JAI_BASE_URL=file:///...
# (the Windows counterpart of tools/ci/test_install_sh.py). Needs no published release.
#
#   powershell -File tools/ci/test_install_ps1.ps1     # from the repository root; Windows PowerShell 5.1 or 7
$ErrorActionPreference = 'Stop'
$temp = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
$base = Join-Path $temp ('install-ps1-' + [guid]::NewGuid().ToString('N'))
$root = Join-Path $base 'release'
$script = Join-Path $PSScriptRoot '..\..\install.ps1'

function Build($version) {
  $stage = Join-Path $base "stage-$version"
  $top = Join-Path $stage 'jacket-windows-x64'
  New-Item -ItemType Directory -Force (Join-Path $top 'stdlib\prelude') | Out-Null
  foreach ($tool in 'jaic', 'jailsp', 'jailint', 'jaifmt') { "stub $version" | Set-Content (Join-Path $top "$tool.exe") }
  $dir = Join-Path $root "v$version"
  New-Item -ItemType Directory -Force $dir | Out-Null
  $zip = Join-Path $dir 'jacket-windows-x64.zip'
  Compress-Archive -Path $top -DestinationPath $zip -Force
  $hash = (Get-FileHash -Algorithm SHA256 $zip).Hash.ToLowerInvariant()
  "$hash  jacket-windows-x64.zip" | Set-Content (Join-Path $dir 'SHA256SUMS')
}

Build '9.0.1'
Build '9.0.2'
$env:JAI_BASE_URL = ([Uri]$root).AbsoluteUri
$env:JAI_INSTALL_DIR = Join-Path $base 'install'
$env:JAI_VERSION = '9.0.1'
Get-Content -Raw $script | Invoke-Expression
$env:JAI_VERSION = '9.0.2'
Get-Content -Raw $script | Invoke-Expression
if (-not (Test-Path (Join-Path $env:JAI_INSTALL_DIR '9.0.2\jaic.exe'))) { throw '9.0.2 was not installed' }
if (Test-Path (Join-Path $env:JAI_INSTALL_DIR '9.0.1')) { throw '9.0.1 was not removed by the upgrade' }
if (-not (Test-Path (Join-Path $env:JAI_INSTALL_DIR 'current\jaic.exe'))) { throw 'the current junction is broken' }
# A damaged download is refused.
Add-Content (Join-Path $root 'v9.0.2\jacket-windows-x64.zip') 'damaged'
$rejected = $false
try { Get-Content -Raw $script | Invoke-Expression } catch { $rejected = "$_" -match 'checksum mismatch' }
if (-not $rejected) { throw 'a damaged archive was not rejected with a checksum error' }
Write-Host 'install.ps1: stub install, upgrade and damaged-archive checks passed'
