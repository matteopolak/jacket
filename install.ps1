# Installs Jacket, the Jai toolchain (jaic, jailsp, jailint, jaifmt) from a GitHub release of
# https://github.com/matteopolak/jacket, verified against the release's SHA256SUMS:
#
#   irm https://raw.githubusercontent.com/matteopolak/jacket/main/install.ps1 | iex
#
# Environment (each JAI_* name also works as JAIC_*; JAI_* wins when both are set):
#   JAI_VERSION      release to install, e.g. 0.4.0 or v0.4.0 (default: the latest release)
#   JAI_INSTALL_DIR  where releases are unpacked (default: %LOCALAPPDATA%\Programs\jai)
#   JAI_BASE_URL     where release assets are fetched from, as <base>/v<version>/<asset> (default:
#                    https://github.com/matteopolak/jai/releases/download). For mirrors and for
#                    testing against local archives with a file:// URL; needs JAI_VERSION.
#
# Each version is unpacked into <dir>\<version>; <dir>\current is a directory junction to the
# installed one and is the folder added to the user PATH. Running it again installs the
# requested (or latest) version, moves the junction and removes the versions it installed before.
# See docs/tools/package-managers.md.

& {
    $ErrorActionPreference = 'Stop'
    # Windows PowerShell 5.1: the progress bar slows downloads down a lot, and TLS 1.2 is opt-in.
    $ProgressPreference = 'SilentlyContinue'
    [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

    $repo = 'matteopolak/jacket'

    $arch = $null
    try { $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString() } catch { }
    if (-not $arch) { $arch = $env:PROCESSOR_ARCHITECTURE }
    switch -regex ($arch) {
        '^(X64|AMD64)$' { $platform = 'windows-x64' }
        '^ARM64$' { $platform = 'windows-arm64' }
        default { throw "There is no prebuilt Jai toolchain for Windows on $arch; see https://github.com/$repo#install" }
    }

    $version = if ($env:JAI_VERSION) { "$env:JAI_VERSION".Trim() } else { "$env:JAIC_VERSION".Trim() }
    if ($version.StartsWith('v')) { $version = $version.Substring(1) }
    $defaultBase = "https://github.com/$repo/releases/download"
    $baseUrl = if ($env:JAI_BASE_URL) { "$env:JAI_BASE_URL".TrimEnd('/') } elseif ($env:JAIC_BASE_URL) { "$env:JAIC_BASE_URL".TrimEnd('/') } else { $defaultBase }
    if (-not $version -or $version -eq 'latest') {
        if ($baseUrl -ne $defaultBase) { throw 'JAI_BASE_URL needs JAI_VERSION' }
        # The /releases/latest page redirects to /releases/tag/v<version>; unlike the API, no
        # rate limit applies.
        $request = [Net.WebRequest]::Create("https://github.com/$repo/releases/latest")
        $request.Method = 'HEAD'
        $request.AllowAutoRedirect = $false
        $request.UserAgent = 'jai-install.ps1'
        $response = $request.GetResponse()
        $location = "$($response.Headers['Location'])"
        $response.Close()
        if ($location -notmatch '/releases/tag/v?([^/]+)$') { throw "Could not find the latest release of $repo (got '$location'); set JAI_VERSION" }
        $version = $Matches[1]
    }
    if ($version -notmatch '^[0-9A-Za-z.-]+$') { throw "JAI_VERSION '$version' is not a version like 0.4.0" }

    $installDir = if ($env:JAI_INSTALL_DIR) { $env:JAI_INSTALL_DIR } elseif ($env:JAIC_INSTALL_DIR) { $env:JAIC_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA 'Programs\jai' }
    $root = "jacket-$platform"
    $archive = "$root.zip"
    $base = "$baseUrl/v$version"
    $tmp = Join-Path ([IO.Path]::GetTempPath()) ("jai-install-" + [Guid]::NewGuid())
    New-Item -ItemType Directory -Force $tmp | Out-Null
    try {
        Write-Host "Downloading jai $version ($archive)"
        foreach ($asset in @('SHA256SUMS', $archive)) {
            if ($base.StartsWith('file://')) { # only reachable through JAI_BASE_URL
                Copy-Item ([Uri]"$base/$asset").LocalPath (Join-Path $tmp $asset)
            } else {
                Invoke-WebRequest -UseBasicParsing "$base/$asset" -OutFile (Join-Path $tmp $asset)
            }
        }
        $expected = $null
        foreach ($line in Get-Content (Join-Path $tmp 'SHA256SUMS')) {
            $fields = $line -split '\s+', 2
            if ($fields.Count -eq 2 -and $fields[1].TrimStart('*') -eq $archive) { $expected = $fields[0].ToLowerInvariant() }
        }
        if (-not $expected) { throw "SHA256SUMS of $version has no entry for $archive" }
        $actual = (Get-FileHash -Algorithm SHA256 (Join-Path $tmp $archive)).Hash.ToLowerInvariant()
        if ($actual -ne $expected) { throw "checksum mismatch for ${archive}: expected $expected, got $actual" }
        Write-Host "Verified SHA-256 $actual"

        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $unpack = Join-Path $tmp 'unpack'
        [IO.Compression.ZipFile]::ExtractToDirectory((Join-Path $tmp $archive), $unpack)
        $unpacked = Join-Path $unpack $root
        if (-not (Test-Path (Join-Path $unpacked 'jaic.exe'))) { throw "$archive does not contain $root\jaic.exe" }

        New-Item -ItemType Directory -Force $installDir | Out-Null
        $target = Join-Path $installDir $version
        $current = Join-Path $installDir 'current'
        # Drop the junction first: removing it never touches the folder it points to.
        if (Test-Path $current) { (Get-Item $current -Force).Delete() }
        if (Test-Path $target) { Remove-Item -Recurse -Force $target }
        Move-Item $unpacked $target
        New-Item -ItemType Junction -Path $current -Target $target | Out-Null
        # Versions installed by earlier runs.
        Get-ChildItem -Directory $installDir | Where-Object {
            $_.Name -match '^[0-9]' -and $_.FullName -ne $target -and (Test-Path (Join-Path $_.FullName 'jaic.exe'))
        } | ForEach-Object {
            Remove-Item -Recurse -Force $_.FullName
            Write-Host "Removed the previous version in $($_.FullName)"
        }
    } finally {
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }

    Write-Host "Installed jai $version into $target"
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $entries = @($userPath -split ';' | Where-Object { $_ })
    if ($entries -notcontains $current) {
        [Environment]::SetEnvironmentVariable('Path', (($entries + $current) -join ';'), 'User')
        Write-Host "Added $current to your user PATH; open a new terminal to use jaic, jailsp, jailint and jaifmt."
    }
    if (($env:Path -split ';') -notcontains $current) { $env:Path = "$env:Path;$current" }
    Write-Host 'Try it: jaic hello.jai -interp'
}
