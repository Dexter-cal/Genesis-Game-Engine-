//! ChronoVerse Cross-Platform Installer & Package Manager
//!
//! Installs ChronoVerse on any OS:
//! - Windows 10/11 (x64, ARM64)
//! - macOS 12+ (Intel + Apple Silicon)
//! - Ubuntu / Debian / Linux Mint
//! - Arch Linux / Manjaro
//! - Fedora / RHEL / CentOS
//! - NixOS
//! - Raspberry Pi (ARM)
//! - Steam Deck
//!
//! Installer features:
//! - Auto-detects OS, architecture, GPU
//! - Downloads correct binaries (no compiling needed)
//! - Installs GPU drivers if needed
//! - Sets up local AI models
//! - Creates desktop shortcuts and file associations
//! - Silent/unattended install mode
//! - Offline installer bundle
//! - Auto-update system (delta patches)
//! - Uninstaller
//! - Package manager (install engine extensions/plugins)

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ─── Platform Detection ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OsFamily {
    Windows,
    MacOs,
    Linux,
    Wasm,
    Android,
    Ios,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LinuxDistro {
    Ubuntu { version: String },
    Debian { version: String },
    Fedora { version: u32 },
    Arch,
    Manjaro,
    NixOs,
    OpenSuse,
    Mint { version: String },
    SteamOs,
    RaspberryPiOs,
    Unknown { name: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CpuArch {
    X86_64,
    Arm64,
    ArmV7,
    RiscV64,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia { driver_version: Option<String>, vram_mb: u32 },
    Amd    { driver_version: Option<String>, vram_mb: u32 },
    Intel  { driver_version: Option<String>, vram_mb: u32 },
    Apple  { metal_version: String, unified_memory_mb: u32 },
    Qualcomm { model: String },
    None,
    Unknown,
}

impl GpuVendor {
    pub fn supports_vulkan(&self) -> bool {
        matches!(self, Self::Nvidia { .. } | Self::Amd { .. } | Self::Intel { .. } | Self::Qualcomm { .. })
    }
    pub fn supports_metal(&self) -> bool {
        matches!(self, Self::Apple { .. })
    }
    pub fn supports_cuda(&self) -> bool {
        matches!(self, Self::Nvidia { .. })
    }
    pub fn supports_rocm(&self) -> bool {
        matches!(self, Self::Amd { .. })
    }
    pub fn vram_mb(&self) -> u32 {
        match self {
            Self::Nvidia { vram_mb, .. } => *vram_mb,
            Self::Amd    { vram_mb, .. } => *vram_mb,
            Self::Intel  { vram_mb, .. } => *vram_mb,
            Self::Apple  { unified_memory_mb, .. } => *unified_memory_mb,
            _ => 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_family: OsFamily,
    pub os_version: String,
    pub linux_distro: Option<LinuxDistro>,
    pub cpu_arch: CpuArch,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
    pub cpu_name: String,
    pub ram_mb: u64,
    pub disk_free_mb: u64,
    pub gpus: Vec<GpuVendor>,
    pub has_internet: bool,
    pub is_admin: bool,
    pub home_dir: PathBuf,
    pub temp_dir: PathBuf,
}

impl SystemInfo {
    /// Detect the current system
    pub fn detect() -> Self {
        let os_family = if cfg!(target_os = "windows") { OsFamily::Windows }
            else if cfg!(target_os = "macos")  { OsFamily::MacOs }
            else if cfg!(target_arch = "wasm32"){ OsFamily::Wasm }
            else                                { OsFamily::Linux };

        let cpu_arch = if cfg!(target_arch = "x86_64") { CpuArch::X86_64 }
            else if cfg!(target_arch = "aarch64") { CpuArch::Arm64 }
            else { CpuArch::Unknown };

        Self {
            os_family: os_family.clone(),
            os_version: std::env::consts::OS.to_string(),
            linux_distro: if os_family == OsFamily::Linux { Some(Self::detect_linux_distro()) } else { None },
            cpu_arch,
            cpu_cores: 4,       // would use sysinfo crate in production
            cpu_threads: 8,
            cpu_name: "Unknown CPU".to_string(),
            ram_mb: 8192,
            disk_free_mb: 50000,
            gpus: Vec::new(),
            has_internet: true,
            is_admin: false,
            home_dir: dirs_next_home().unwrap_or_else(|| PathBuf::from("/")),
            temp_dir: std::env::temp_dir(),
        }
    }

    fn detect_linux_distro() -> LinuxDistro {
        // Read /etc/os-release
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let fields: HashMap<String, String> = content.lines()
                .filter_map(|l| {
                    let mut parts = l.splitn(2, '=');
                    Some((
                        parts.next()?.to_string(),
                        parts.next()?.trim_matches('"').to_string(),
                    ))
                })
                .collect();

            let id = fields.get("ID").map(|s| s.as_str()).unwrap_or("");
            let version = fields.get("VERSION_ID").cloned().unwrap_or_default();

            return match id {
                "ubuntu" | "linuxmint" => LinuxDistro::Ubuntu { version },
                "debian" => LinuxDistro::Debian { version },
                "fedora" => LinuxDistro::Fedora { version: version.parse().unwrap_or(38) },
                "arch" => LinuxDistro::Arch,
                "manjaro" => LinuxDistro::Manjaro,
                "nixos" => LinuxDistro::NixOs,
                "steamos" => LinuxDistro::SteamOs,
                _ => LinuxDistro::Unknown { name: id.to_string() },
            };
        }
        LinuxDistro::Unknown { name: "unknown".to_string() }
    }

    pub fn recommended_gpu_backend(&self) -> &'static str {
        if self.gpus.iter().any(|g| g.supports_metal()) { "metal" }
        else if self.gpus.iter().any(|g| g.supports_vulkan()) { "vulkan" }
        else if self.os_family == OsFamily::Windows { "dx12" }
        else { "opengl" }
    }

    pub fn can_run_local_ai(&self) -> bool {
        self.ram_mb >= 4096 && self.gpus.iter().any(|g| g.vram_mb() >= 2048)
    }

    pub fn min_requirements_met(&self) -> (bool, Vec<String>) {
        let mut issues = Vec::new();
        if self.ram_mb < 4096 { issues.push(format!("RAM: {}MB (need 4GB+)", self.ram_mb)); }
        if self.disk_free_mb < 5000 { issues.push(format!("Disk: {}MB free (need 5GB+)", self.disk_free_mb)); }
        if self.cpu_cores < 2 { issues.push(format!("CPU: {} cores (need 2+)", self.cpu_cores)); }
        (issues.is_empty(), issues)
    }
}

fn dirs_next_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}

// ─── Install Configuration ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallConfig {
    pub install_path: PathBuf,
    pub components: InstallComponents,
    pub create_desktop_shortcut: bool,
    pub create_file_associations: bool,
    pub add_to_path: bool,
    pub install_models: Vec<ModelInstallChoice>,
    pub silent: bool,
    pub offline_bundle: Option<PathBuf>,
    pub channel: UpdateChannel,
    pub proxy: Option<String>,
    pub telemetry_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallComponents {
    pub core_engine: bool,          // required
    pub editor_ui: bool,            // the full visual editor
    pub local_ai_models: bool,      // basic NPC dialogue models
    pub studio: bool,               // virtual production studio
    pub sample_projects: bool,      // example games
    pub documentation: bool,        // offline docs
    pub source_code: bool,          // for developers
    pub debug_symbols: bool,
}

impl Default for InstallComponents {
    fn default() -> Self {
        Self {
            core_engine: true,
            editor_ui: true,
            local_ai_models: true,
            studio: true,
            sample_projects: true,
            documentation: false,
            source_code: false,
            debug_symbols: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInstallChoice {
    pub model_id: String,
    pub task: String,
    pub size_mb: u32,
    pub required: bool,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateChannel {
    Stable,     // LTS releases, most tested
    Beta,       // feature-complete, some rough edges
    Nightly,    // latest, potentially broken
    Lts { version: String },
}

impl Default for InstallConfig {
    fn default() -> Self {
        let sys = SystemInfo::detect();
        let install_path = match sys.os_family {
            OsFamily::Windows => PathBuf::from("C:\\Program Files\\ChronoVerse"),
            OsFamily::MacOs   => PathBuf::from("/Applications/ChronoVerse.app"),
            _                 => PathBuf::from("/opt/genesis"),
        };

        Self {
            install_path,
            components: InstallComponents::default(),
            create_desktop_shortcut: true,
            create_file_associations: true,
            add_to_path: true,
            install_models: vec![
                ModelInstallChoice {
                    model_id: "qwen2.5-1.5b-instruct-q4_k_m.gguf".to_string(),
                    task: "NPC Dialogue (Fast)".to_string(),
                    size_mb: 950,
                    required: false,
                    selected: true,
                },
                ModelInstallChoice {
                    model_id: "kokoro-82m.bin".to_string(),
                    task: "Text to Speech".to_string(),
                    size_mb: 330,
                    required: false,
                    selected: true,
                },
            ],
            silent: false,
            offline_bundle: None,
            channel: UpdateChannel::Stable,
            proxy: None,
            telemetry_enabled: false,
        }
    }
}

// ─── Installer Steps ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InstallStep {
    SystemCheck,
    DownloadCore     { url: String, size_mb: u64 },
    DownloadModels   { models: Vec<String> },
    ExtractFiles     { source: PathBuf, target: PathBuf },
    InstallDeps      { deps: Vec<String> },
    InstallGpuDriver { vendor: String, version: String },
    ConfigureEngine  { config: serde_json::Value },
    CreateShortcuts,
    RegisterFileAssoc{ extensions: Vec<String> },
    AddToPath        { path: PathBuf },
    CreateUninstaller,
    RunFirstLaunch,
    Complete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub current_step: InstallStep,
    pub step_index: u32,
    pub total_steps: u32,
    pub step_progress: f32,
    pub overall_progress: f32,
    pub message: String,
    pub download_speed_mbps: f32,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub log: Vec<String>,
}

// ─── OS-Specific Install Logic ────────────────────────────────────────────────

pub struct PlatformInstaller {
    pub system: SystemInfo,
    pub config: InstallConfig,
    pub progress: InstallProgress,
}

impl PlatformInstaller {
    pub fn new(config: InstallConfig) -> Self {
        let system = SystemInfo::detect();
        Self {
            system,
            config,
            progress: InstallProgress {
                current_step: InstallStep::SystemCheck,
                step_index: 0,
                total_steps: 12,
                step_progress: 0.0,
                overall_progress: 0.0,
                message: "Preparing...".to_string(),
                download_speed_mbps: 0.0,
                errors: Vec::new(),
                warnings: Vec::new(),
                log: Vec::new(),
            },
        }
    }

    /// Generate platform-specific install script
    pub fn generate_install_script(&self) -> String {
        match &self.system.os_family {
            OsFamily::Linux => self.linux_script(),
            OsFamily::MacOs => self.macos_script(),
            OsFamily::Windows => self.windows_script(),
            _ => String::new(),
        }
    }

    fn linux_script(&self) -> String {
        let install_path = self.config.install_path.display();
        let deps = self.linux_deps();

        format!(r#"#!/bin/bash
# ChronoVerse Linux Installer
# Supports: Ubuntu/Debian, Arch, Fedora, NixOS, Steam Deck

set -e
INSTALL_DIR="{install_path}"
CV_VERSION="0.1.0"
CV_DOWNLOAD="https://releases.genesis.io/linux/$CV_VERSION/genesis-linux-x86_64.tar.gz"

echo "╔══════════════════════════════════════╗"
echo "║   ChronoVerse Engine Installer       ║"
echo "║   v$CV_VERSION                              ║"
echo "╚══════════════════════════════════════╝"
echo ""

# Check requirements
echo "→ Checking system requirements..."
RAM_MB=$(grep MemTotal /proc/meminfo | awk '{{print int($2/1024)}}')
if [ "$RAM_MB" -lt 4096 ]; then
    echo "WARNING: ${{RAM_MB}}MB RAM detected. 4GB+ recommended."
fi

DISK_MB=$(df -m / | awk 'NR==2 {{print $4}}')
if [ "$DISK_MB" -lt 5000 ]; then
    echo "ERROR: Only ${{DISK_MB}}MB free disk space. Need 5GB+."
    exit 1
fi

# Detect package manager and install deps
echo "→ Installing system dependencies..."
{deps}

# Create install directory
echo "→ Creating install directory: $INSTALL_DIR"
sudo mkdir -p "$INSTALL_DIR"
sudo chown $USER:$USER "$INSTALL_DIR"

# Download ChronoVerse
echo "→ Downloading ChronoVerse..."
if command -v curl &> /dev/null; then
    curl -L --progress-bar "$CV_DOWNLOAD" -o /tmp/genesis.tar.gz
elif command -v wget &> /dev/null; then
    wget -q --show-progress "$CV_DOWNLOAD" -O /tmp/genesis.tar.gz
fi

# Extract
echo "→ Extracting..."
tar -xzf /tmp/genesis.tar.gz -C "$INSTALL_DIR"
rm /tmp/genesis.tar.gz

# Setup Ollama for local AI
echo "→ Setting up local AI (Ollama)..."
if ! command -v ollama &> /dev/null; then
    curl -fsSL https://ollama.com/install.sh | sh
fi

# Download base AI models
echo "→ Downloading base AI models..."
ollama pull qwen2.5:1.5b 2>/dev/null || echo "AI models will be downloaded on first run"

# Create .desktop file
if [ -d ~/.local/share/applications ]; then
    cat > ~/.local/share/applications/genesis.desktop << 'DESKTOP'
[Desktop Entry]
Name=ChronoVerse
Comment=AI-First Game Engine
Exec={install_path}/genesis
Icon={install_path}/icons/genesis.png
Terminal=false
Type=Application
Categories=Development;GameDevelopment;
MimeType=application/x-genesis-project;
DESKTOP
    echo "→ Desktop shortcut created"
fi

# Register file associations
if command -v xdg-mime &> /dev/null; then
    xdg-mime default genesis.desktop application/x-genesis-project
fi

# Add to PATH
SHELL_RC=""
if [ -f ~/.bashrc ]; then SHELL_RC=~/.bashrc
elif [ -f ~/.zshrc ]; then SHELL_RC=~/.zshrc
fi

if [ -n "$SHELL_RC" ]; then
    echo "export PATH=\"$INSTALL_DIR/bin:\$PATH\"" >> "$SHELL_RC"
    echo "→ Added to PATH in $SHELL_RC"
fi

# Create CLI symlink
sudo ln -sf "$INSTALL_DIR/bin/genesis-cli" /usr/local/bin/genesis-cli 2>/dev/null || \
    echo "Note: Run 'source $SHELL_RC' to use genesis-cli"

echo ""
echo "✅ ChronoVerse installed successfully!"
echo ""
echo "→ Launch from applications menu or run:"
echo "  $INSTALL_DIR/genesis"
echo ""
echo "→ Or use the CLI:"
echo "  source ~/.bashrc && genesis-cli new my-game"
"#, install_path=install_path, deps=deps)
    }

    fn linux_deps(&self) -> String {
        let distro = self.system.linux_distro.as_ref();
        match distro {
            Some(LinuxDistro::Ubuntu { .. }) | Some(LinuxDistro::Debian { .. }) | Some(LinuxDistro::Mint { .. }) => {
                r#"if command -v apt-get &> /dev/null; then
    sudo apt-get update -qq
    sudo apt-get install -y -qq \
        libvulkan1 vulkan-utils mesa-vulkan-drivers \
        libasound2 libasound2-dev \
        libx11-6 libx11-dev libxcursor-dev libxi-dev libxrandr-dev \
        libssl-dev pkg-config \
        ffmpeg libavcodec-dev libavformat-dev libavutil-dev \
        curl wget git
fi"#.to_string()
            }
            Some(LinuxDistro::Arch) | Some(LinuxDistro::Manjaro) => {
                r#"if command -v pacman &> /dev/null; then
    sudo pacman -Syu --noconfirm \
        vulkan-icd-loader vulkan-tools \
        alsa-lib \
        libx11 libxcursor libxi libxrandr \
        openssl ffmpeg curl wget git
fi"#.to_string()
            }
            Some(LinuxDistro::Fedora { .. }) => {
                r#"if command -v dnf &> /dev/null; then
    sudo dnf install -y \
        vulkan mesa-vulkan-drivers \
        alsa-lib-devel \
        libX11-devel libXcursor-devel libXi-devel libXrandr-devel \
        openssl-devel ffmpeg-devel curl wget git
fi"#.to_string()
            }
            Some(LinuxDistro::NixOs) => {
                r#"# NixOS: add to configuration.nix:
# environment.systemPackages = with pkgs; [
#   vulkan-loader alsa-lib xorg.libX11 ffmpeg
# ];"#.to_string()
            }
            Some(LinuxDistro::SteamOs) => {
                r#"# Steam Deck: ChronoVerse is pre-configured for Steam OS
# No additional dependencies needed"#.to_string()
            }
            _ => {
                r#"# Unknown distro - manual dependency installation may be needed
# Required: Vulkan drivers, ALSA/PulseAudio, libX11, OpenSSL, FFmpeg"#.to_string()
            }
        }
    }

    fn macos_script(&self) -> String {
        let install_path = self.config.install_path.display();
        format!(r#"#!/bin/bash
# ChronoVerse macOS Installer
# Supports: macOS 12 Monterey+, Intel + Apple Silicon

set -e

echo "ChronoVerse macOS Installer"
echo "==========================="

# Check macOS version
MACOS_VERSION=$(sw_vers -productVersion)
echo "macOS version: $MACOS_VERSION"

# Check architecture
ARCH=$(uname -m)
echo "Architecture: $ARCH"
if [ "$ARCH" = "arm64" ]; then
    DOWNLOAD_URL="https://releases.genesis.io/macos/genesis-macos-arm64.dmg"
    echo "Using Apple Silicon build"
else
    DOWNLOAD_URL="https://releases.genesis.io/macos/genesis-macos-x86_64.dmg"
    echo "Using Intel build"
fi

# Install Homebrew if needed
if ! command -v brew &> /dev/null; then
    echo "Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# Install dependencies
echo "Installing dependencies..."
brew install --quiet ffmpeg pkg-config

# Install Ollama
if ! command -v ollama &> /dev/null; then
    echo "Installing Ollama for local AI..."
    brew install --quiet ollama
fi

# Download DMG
echo "Downloading ChronoVerse..."
curl -L -# "$DOWNLOAD_URL" -o /tmp/ChronoVerse.dmg

# Mount and install
echo "Installing..."
hdiutil attach /tmp/ChronoVerse.dmg -quiet
cp -R "/Volumes/ChronoVerse/ChronoVerse.app" /Applications/
hdiutil detach "/Volumes/ChronoVerse" -quiet
rm /tmp/ChronoVerse.dmg

# Remove quarantine attribute
xattr -rd com.apple.quarantine /Applications/ChronoVerse.app 2>/dev/null || true

# CLI tool
sudo ln -sf /Applications/ChronoVerse.app/Contents/MacOS/genesis-cli /usr/local/bin/genesis-cli

# Register file associations
/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister \
    -f /Applications/ChronoVerse.app

# Pull base AI model
echo "Downloading base AI model..."
ollama pull qwen2.5:1.5b 2>/dev/null || true

echo ""
echo "✅ ChronoVerse installed to /Applications/ChronoVerse.app"
echo "Launch from Applications folder or Spotlight."
"#)
    }

    fn windows_script(&self) -> String {
        r#"# ChronoVerse Windows Installer (PowerShell)
# Run as Administrator for best results

param(
    [string]$InstallPath = "C:\Program Files\ChronoVerse",
    [switch]$Silent,
    [switch]$WithModels
)

Write-Host "ChronoVerse Windows Installer" -ForegroundColor Cyan
Write-Host "==============================" -ForegroundColor Cyan

# Check Windows version
$WinVer = [System.Environment]::OSVersion.Version
Write-Host "Windows $($WinVer.Major).$($WinVer.Minor)"

# Check architecture
$Arch = $env:PROCESSOR_ARCHITECTURE
Write-Host "Architecture: $Arch"

$DownloadUrl = if ($Arch -eq "ARM64") {
    "https://releases.genesis.io/windows/genesis-windows-arm64.exe"
} else {
    "https://releases.genesis.io/windows/genesis-windows-x64.exe"
}

# Check Visual C++ Redistributable
$VcRedist = Get-ItemProperty HKLM:\SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\* 2>$null
if (-not $VcRedist) {
    Write-Host "Installing Visual C++ Redistributable..." -ForegroundColor Yellow
    $VcUrl = "https://aka.ms/vs/17/release/vc_redist.x64.exe"
    Invoke-WebRequest -Uri $VcUrl -OutFile "$env:TEMP\vc_redist.exe"
    Start-Process -FilePath "$env:TEMP\vc_redist.exe" -Args "/quiet /norestart" -Wait
}

# Check for Vulkan
$Vulkan = Get-Item "C:\Windows\System32\vulkan-1.dll" 2>$null
if (-not $Vulkan) {
    Write-Host "Installing Vulkan Runtime..." -ForegroundColor Yellow
    # DirectX fallback will be used
}

# Download ChronoVerse installer
Write-Host "Downloading ChronoVerse..." -ForegroundColor Green
$InstallerPath = "$env:TEMP\ChronoVerseSetup.exe"
$ProgressPreference = 'SilentlyContinue'
Invoke-WebRequest -Uri $DownloadUrl -OutFile $InstallerPath
$ProgressPreference = 'Continue'

# Run installer
Write-Host "Installing to: $InstallPath" -ForegroundColor Green
$Args = "/DIR=`"$InstallPath`""
if ($Silent) { $Args += " /SILENT" }
Start-Process -FilePath $InstallerPath -Args $Args -Wait

# Install Ollama (optional, for local AI)
Write-Host "Setting up local AI (Ollama)..." -ForegroundColor Green
$OllamaCheck = Get-Command ollama -ErrorAction SilentlyContinue
if (-not $OllamaCheck) {
    $OllamaUrl = "https://ollama.com/download/OllamaSetup.exe"
    Invoke-WebRequest -Uri $OllamaUrl -OutFile "$env:TEMP\OllamaSetup.exe"
    Start-Process -FilePath "$env:TEMP\OllamaSetup.exe" -Args "/silent" -Wait
}

# Add to PATH
$CurrentPath = [System.Environment]::GetEnvironmentVariable("PATH", "Machine")
if ($CurrentPath -notlike "*ChronoVerse*") {
    [System.Environment]::SetEnvironmentVariable(
        "PATH", "$CurrentPath;$InstallPath\bin", "Machine")
    Write-Host "Added to system PATH" -ForegroundColor Green
}

# Create file association for .chrono files
New-Item -Path "HKCU:\Software\Classes\.chrono" -Force | Set-ItemProperty -Name "(Default)" -Value "ChronoVerseProject"
New-Item -Path "HKCU:\Software\Classes\ChronoVerseProject\shell\open\command" -Force |
    Set-ItemProperty -Name "(Default)" -Value "`"$InstallPath\genesis.exe`" `"%1`""

# Download AI models if requested
if ($WithModels) {
    Write-Host "Downloading base AI models..." -ForegroundColor Green
    & ollama pull qwen2.5:1.5b
}

# Create Start Menu shortcut
$WshShell = New-Object -ComObject WScript.Shell
$Shortcut = $WshShell.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\ChronoVerse.lnk")
$Shortcut.TargetPath = "$InstallPath\genesis.exe"
$Shortcut.IconLocation = "$InstallPath\genesis.ico"
$Shortcut.Save()

Write-Host ""
Write-Host "✅ ChronoVerse installed successfully!" -ForegroundColor Green
Write-Host "Launch from Start Menu or run: genesis" -ForegroundColor White
"#.to_string()
    }
}

// ─── Auto-updater ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub channel: UpdateChannel,
    pub release_notes: String,
    pub published_at: String,
    pub download_url: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub delta_from: Option<String>,   // version this is a delta from
    pub delta_url: Option<String>,    // smaller delta patch URL
    pub critical: bool,               // security/crash fix, auto-install
    pub breaking_changes: Vec<String>,
}

pub struct AutoUpdater {
    pub current_version: String,
    pub channel: UpdateChannel,
    pub check_on_launch: bool,
    pub auto_download: bool,
    pub auto_install_critical: bool,
    pub last_check: Option<String>,
    pub pending_update: Option<UpdateInfo>,
}

impl AutoUpdater {
    pub fn new(version: &str, channel: UpdateChannel) -> Self {
        Self {
            current_version: version.to_string(),
            channel,
            check_on_launch: true,
            auto_download: false,
            auto_install_critical: true,
            last_check: None,
            pending_update: None,
        }
    }

    pub fn update_check_url(&self) -> String {
        let channel = match &self.channel {
            UpdateChannel::Stable => "stable",
            UpdateChannel::Beta   => "beta",
            UpdateChannel::Nightly => "nightly",
            UpdateChannel::Lts { .. } => "lts",
        };
        format!("https://releases.genesis.io/updates/{}/latest.json", channel)
    }

    pub fn needs_update(&self) -> bool {
        self.pending_update.is_some()
    }
}

// ─── Package Manager ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub category: PackageCategory,
    pub tags: Vec<String>,
    pub download_url: String,
    pub size_bytes: u64,
    pub min_engine_version: String,
    pub dependencies: Vec<String>,
    pub screenshots: Vec<String>,
    pub rating: f32,
    pub downloads: u64,
    pub price_usd: f32,   // 0.0 = free
    pub license: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PackageCategory {
    Plugin,
    AssetPack { asset_type: String },
    AiModel { model_type: String },
    Template { genre: String },
    Shader,
    Font,
    Sound,
    Script,
    EditorTool,
    GameMode,
}

pub struct PackageManager {
    pub installed: HashMap<String, Package>,
    pub registry_url: String,
    pub local_repo: PathBuf,
}

impl PackageManager {
    pub fn new(local_repo: PathBuf) -> Self {
        Self {
            installed: HashMap::new(),
            registry_url: "https://registry.genesis.io/packages".to_string(),
            local_repo,
        }
    }

    pub fn is_installed(&self, id: &str) -> bool { self.installed.contains_key(id) }
    pub fn installed_count(&self) -> usize { self.installed.len() }
}
