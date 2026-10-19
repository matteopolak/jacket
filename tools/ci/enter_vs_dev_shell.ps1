# Dot-source this from a pwsh step (`. tools/ci/enter_vs_dev_shell.ps1`) to put the Visual Studio C++
# environment for $env:VS_ARCH (x64 or arm64) into the current session: LIB, INCLUDE and PATH, as the
# native SDK checks need them.
$ErrorActionPreference = 'Stop'
$installation = & "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe" -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if (-not $installation) { throw 'Visual Studio native SDK tools are required' }
& (Join-Path $installation 'Common7/Tools/Launch-VsDevShell.ps1') -Arch $env:VS_ARCH -HostArch amd64 -SkipAutomaticLocation
