param(
    [ValidateSet("Validate", "Build", "Run", "Probe", "OfflineTests")]
    [string]$Action = "Validate"
)

$ErrorActionPreference = "Stop"
if (-not $IsWindows) { throw "Exécuter avec PowerShell Windows natif." }
$root = Split-Path $PSScriptRoot -Parent
if ($root.StartsWith("\\")) { throw "Utiliser une copie locale Windows, pas le chemin UNC WSL." }
Set-Location $root

# Cargo integration tests can rebuild/remove the ordinary application binary,
# not just deps/* test binaries. Protect an active note BEFORE invoking Cargo.
if ($Action -in @("Validate", "Build")) {
    $appExe = Join-Path $root "src-tauri\target\debug\vocalnotes.exe"
    foreach ($process in @(Get-Process -Name vocalnotes -ErrorAction SilentlyContinue)) {
        if (-not $process.Path) { throw "Identité du processus vocalnotes inconnue ; compilation suspendue." }
        if ($process.Path -eq $appExe) {
            throw "Application active PID $($process.Id) : récupérer/sauvegarder le texte et confirmer la fermeture avant Validate/Build. Aucun arrêt automatique."
        }
    }
}

# Process-local environment only. Never install a toolchain implicitly.
$env:RUSTUP_AUTO_INSTALL = "0"
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$nodeBin = "C:\Program Files\nodejs"
$env:PATH = "$cargoBin;$nodeBin;$env:PATH"
$cargo = Join-Path $cargoBin "cargo.exe"
$rustc = Join-Path $cargoBin "rustc.exe"
$npm = Join-Path $nodeBin "npm.cmd"
foreach ($tool in @($cargo, $rustc, $npm)) {
    if (-not (Test-Path -LiteralPath $tool -PathType Leaf)) { throw "Outil absent : $tool. Installation utilisateur requise." }
}

function Invoke-Checked {
    param([string]$Program, [string[]]$Arguments)
    & $Program @Arguments
    $code = $LASTEXITCODE
    if ($code -ne 0) { throw "Échec ($code) : $Program $($Arguments -join ' ')" }
}

if ($Action -eq "Run") {
    $exe = Join-Path $root "src-tauri\target\debug\vocalnotes.exe"
    if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) { throw "Compiler d'abord avec -Action Build." }
    # The caller owns this foreground process tree; no Start-Process, no hidden daemon.
    Invoke-Checked $exe @()
    exit 0
}

$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path -LiteralPath $vswhere)) { throw "vswhere absent ; vérifier Visual Studio Build Tools." }
$vs = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or -not $vs) { throw "Build Tools C++ x64 non détectés." }
Import-Module (Join-Path $vs "Common7\Tools\Microsoft.VisualStudio.DevShell.dll")
Enter-VsDevShell -VsInstallPath $vs -SkipAutomaticLocation -DevCmdArguments "-arch=x64 -host_arch=x64"
Set-Location $root
Get-Command cl.exe, link.exe, rc.exe | ForEach-Object { "$($_.Name)=$($_.Source)" }
"SDK=$env:WindowsSDKVersion"
Invoke-Checked $rustc @("+1.99.1", "-vV")
Invoke-Checked $cargo @("+1.99.1", "-V")
Invoke-Checked (Join-Path $nodeBin "node.exe") @("--version")
Invoke-Checked $npm @("--version")

if ($Action -eq "Probe") {
    # WTS initial state/subscription only: no microphone, keyring or provider call.
    Invoke-Checked $cargo @("+1.99.1", "run", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--example", "probe_lifecycle")
    exit 0
}

if ($Action -eq "OfflineTests") {
    # Bounded checks with existing tools only. No capture/provider/probe, npm ci,
    # auto-install, ordinary application link or restart. Integration executables
    # reuse the actual test files through an example harness, not integration
    # targets (even cargo rustc --test implicitly builds the ordinary binary).
    $appExe = Join-Path $root "src-tauri\target\debug\vocalnotes.exe"
    $exeGuard = $null
    try {
    if (Test-Path -LiteralPath $appExe) {
        # Deny write/delete for the entire bounded check, not merely detect an
        # accidental overwrite afterwards. This is NOT a lock on a user note.
        $exeGuard = [System.IO.File]::Open($appExe, [System.IO.FileMode]::Open,
            [System.IO.FileAccess]::Read, [System.IO.FileShare]::Read)
    }
    $beforeHash = if (Test-Path -LiteralPath $appExe) {
        (Get-FileHash -LiteralPath $appExe -Algorithm SHA256).Hash
    } else { $null }
    if (-not (Test-Path "node_modules\.bin\vitest.cmd")) {
        throw "Vitest local absent ; aucune installation en mode hors ligne."
    }
    Invoke-Checked $npm @("test")
    Invoke-Checked $cargo @("+1.99.1", "fmt", "--manifest-path", "src-tauri/Cargo.toml", "--", "--check")
    Invoke-Checked $cargo @("+1.99.1", "test", "--offline", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--lib")
    Invoke-Checked $cargo @("+1.99.1", "test", "--offline", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--example", "offline_contract_tests")
    Invoke-Checked $cargo @("+1.99.1", "check", "--offline", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--lib", "--features", "tauri/custom-protocol")
    $afterHash = if (Test-Path -LiteralPath $appExe) {
        (Get-FileHash -LiteralPath $appExe -Algorithm SHA256).Hash
    } else { $null }
    if ($beforeHash -ne $afterHash) { throw "L'exécutable application a changé pendant les tests ; vérifier avant toute recette." }
    "OFFLINE TESTS PASS ; exécutable application inchangé."
    exit 0
    } finally {
        if ($exeGuard) { $exeGuard.Dispose() }
    }
}

if (-not (Test-Path "node_modules\.bin\vite.cmd")) {
    Invoke-Checked $npm @("ci", "--no-audit", "--no-fund")
}
Invoke-Checked $npm @("test")
Invoke-Checked $npm @("run", "build")
Invoke-Checked $cargo @("+1.99.1", "fmt", "--manifest-path", "src-tauri/Cargo.toml", "--", "--check")
Invoke-Checked $cargo @("+1.99.1", "test", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--tests")
Invoke-Checked $cargo @("+1.99.1", "check", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--tests")
if ($Action -eq "Build") {
    # Embedded local assets, not devUrl/Vite and not a Linux cross-compilation.
    # No installer generation: WebView2 must already be provided by the user.
    Invoke-Checked $cargo @("+1.99.1", "build", "--locked", "--manifest-path", "src-tauri/Cargo.toml", "--features", "tauri/custom-protocol")
    Get-Item "src-tauri\target\debug\vocalnotes.exe" | Select-Object FullName, Length, LastWriteTime
}
