param(
    [ValidateSet("Probe", "Validate")]
    [string]$Action = "Validate"
)
$ErrorActionPreference = "Stop"
if (-not $IsWindows) { throw "Windows natif requis." }
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root
$env:RUSTUP_AUTO_INSTALL = "0"
$cargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
$npm = "C:\Program Files\nodejs\npm.cmd"
$env:PATH = "$(Join-Path $env:USERPROFILE '.cargo\bin');C:\Program Files\nodejs;$env:PATH"
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or -not $vs) { throw "Build Tools manquants." }
Import-Module (Join-Path $vs "Common7\Tools\Microsoft.VisualStudio.DevShell.dll")
Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments "-arch=x64 -host_arch=x64"
Set-Location $root
function Checked {
    param([string]$Program, [string[]]$Arguments)
    & $Program @Arguments
    if ($LASTEXITCODE -ne 0) { throw "Échec de validation : code $LASTEXITCODE." }
}
if ($Action -eq "Probe") {
    # Exactly one no-key handshake, no capture or retry; never run the application.
    Checked $cargo @("+1.99.1", "run", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--example", "probe_stt_transport")
    exit 0
}
Checked $npm @("test")
Checked $npm @("run", "build")
Checked $cargo @("+1.99.1", "fmt", "--manifest-path", "src-tauri/Cargo.toml", "--", "--check")
# Library-only targets never replace/remove the active application executable.
Checked $cargo @("+1.99.1", "test", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--lib")
Checked $cargo @("+1.99.1", "check", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--lib", "--features", "tauri/custom-protocol")
Checked $cargo @("+1.99.1", "build", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--lib", "--features", "tauri/custom-protocol")
