# Runs `winget validate` on a folder of manifests and fails on any error or warning, except the
# one an older winget gives for a schema newer than itself ("The schema header URL does not match
# the expected pattern"): the templates use the schema komac submits with, which may be newer than
# the runner's winget. See docs/tools/package-managers.md.
param([Parameter(Mandatory = $true)][string]$Manifests)

winget --version
$output = winget validate --manifest $Manifests | Out-String
$code = $LASTEXITCODE
Write-Host $output
# APPINSTALLER_CLI_ERROR_MANIFEST_VALIDATION_WARNING: valid, with warnings.
$warningsOnly = -1978335192
$problems = @($output -split "`r?`n" | Where-Object {
        $_ -match 'Manifest (Error|Warning)' -and $_ -notmatch 'schema header URL does not match the expected pattern'
    })
if (($code -ne 0 -and $code -ne $warningsOnly) -or $problems.Count -gt 0) {
    throw "winget validate failed (exit code $code)"
}
# winget's own exit code would otherwise become the step's.
exit 0
