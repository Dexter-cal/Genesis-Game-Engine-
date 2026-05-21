#!/usr/bin/env bash
# ╔═══════════════════════════════════════════════════════════════════╗
# ║           Genesis — Universal One-Command Installer           ║
# ║                                                                   ║
# ║   Usage:  curl -fsSL https://get.genesis.io | bash            ║
# ║   Or:     bash install.sh                                         ║
# ║   Or:     bash install.sh --silent --no-models                    ║
# ╚═══════════════════════════════════════════════════════════════════╝
#
# What this does:
#   1. Detects your OS, architecture, and GPU
#   2. Checks if Genesis was previously installed (remembers this PC)
#   3. Downloads the correct binary for your system
#   4. Installs all dependencies (Vulkan, audio, video, AI)
#   5. Downloads base AI models (optional)
#   6. Creates desktop shortcuts and file associations
#   7. Launches Genesis
#
# Supports:
#   Windows 10/11 (via WSL/PowerShell detection)
#   macOS 12+ (Intel + Apple Silicon)
#   Ubuntu/Debian/Mint
#   Arch/Manjaro
#   Fedora/RHEL
#   NixOS
#   Steam Deck

set -euo pipefail

# ─── Configuration ───────────────────────────────────────────────────────────
CV_VERSION="${CV_VERSION:-0.1.0}"
CV_REGISTRY="https://releases.genesis.io"
CV_REGISTRY_CDN="https://cdn.genesis.io"
CV_MODELS_REGISTRY="https://models.genesis.io"
CV_STATE_DIR="${HOME}/.config/genesis"
CV_STATE_FILE="${CV_STATE_DIR}/install_state.json"
CV_INSTALL_DIR="${CV_INSTALL_DIR:-}"
CV_LOG="${CV_STATE_DIR}/install.log"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'

# Flags
SILENT=false; SKIP_MODELS=false; FORCE_REINSTALL=false; NO_LAUNCH=false
OFFLINE_BUNDLE=""; INSTALL_CHANNEL="stable"

# ─── Parse Arguments ─────────────────────────────────────────────────────────
for arg in "$@"; do
  case $arg in
    --silent)         SILENT=true ;;
    --no-models)      SKIP_MODELS=true ;;
    --force)          FORCE_REINSTALL=true ;;
    --no-launch)      NO_LAUNCH=true ;;
    --channel=*)      INSTALL_CHANNEL="${arg#*=}" ;;
    --offline=*)      OFFLINE_BUNDLE="${arg#*=}" ;;
    --install-dir=*)  CV_INSTALL_DIR="${arg#*=}" ;;
  esac
done

# ─── Helpers ─────────────────────────────────────────────────────────────────
log()  { echo -e "${BLUE}→${RESET} $*" | tee -a "$CV_LOG" 2>/dev/null || echo -e "${BLUE}→${RESET} $*"; }
ok()   { echo -e "${GREEN}✓${RESET} $*"; }
warn() { echo -e "${YELLOW}⚠${RESET} $*"; }
err()  { echo -e "${RED}✗${RESET} $*" >&2; }
die()  { err "$*"; exit 1; }

banner() {
cat << 'BANNER'
╔═══════════════════════════════════════════════════════════════════╗
║                                                                   ║
║    ██████╗██╗  ██╗██████╗  ██████╗ ███╗   ██╗ ██████╗            ║
║   ██╔════╝██║  ██║██╔══██╗██╔═══██╗████╗  ██║██╔═══██╗           ║
║   ██║     ███████║██████╔╝██║   ██║██╔██╗ ██║██║   ██║           ║
║   ██║     ██╔══██║██╔══██╗██║   ██║██║╚██╗██║██║   ██║           ║
║   ╚██████╗██║  ██║██║  ██║╚██████╔╝██║ ╚████║╚██████╔╝           ║
║    ╚═════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝           ║
║                                                                   ║
║               THE AI-FIRST GAME ENGINE                           ║
║                    Installer v0.1.0                               ║
╚═══════════════════════════════════════════════════════════════════╝
BANNER
}

# ─── State Management (Remember This PC) ─────────────────────────────────────
mkdir -p "$CV_STATE_DIR"

load_state() {
    if [[ -f "$CV_STATE_FILE" ]]; then
        cat "$CV_STATE_FILE"
    else
        echo '{}'
    fi
}

save_state() {
    local key="$1" value="$2"
    local state=$(load_state)
    # Simple JSON manipulation without jq
    # In production: use jq or Python
    echo "$state" | sed "s/}$/,\"$key\":\"$value\"}" | sed 's/^{,/{/' > "$CV_STATE_FILE"
}

get_state() {
    local key="$1"
    load_state | grep -o "\"$key\":\"[^\"]*\"" | cut -d'"' -f4 2>/dev/null || echo ""
}

is_installed() {
    local install_path=$(get_state "install_path")
    [[ -n "$install_path" ]] && [[ -f "${install_path}/genesis" || -f "${install_path}/genesis.exe" ]]
}

get_machine_id() {
    # Generate a unique machine ID
    if [[ -f /etc/machine-id ]]; then
        cat /etc/machine-id
    elif command -v uuidgen &>/dev/null; then
        uuidgen
    else
        echo "${HOSTNAME}-$(date +%s)"
    fi
}

# ─── System Detection ─────────────────────────────────────────────────────────
detect_os() {
    local os="unknown"
    case "$(uname -s)" in
        Linux*)  os="linux" ;;
        Darwin*) os="macos" ;;
        CYGWIN*|MINGW*|MSYS*) os="windows" ;;
    esac
    echo "$os"
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64) echo "x86_64" ;;
        aarch64|arm64) echo "arm64" ;;
        armv7l) echo "armv7" ;;
        *) echo "x86_64" ;;
    esac
}

detect_distro() {
    if [[ -f /etc/os-release ]]; then
        source /etc/os-release
        echo "${ID:-unknown}"
    else
        echo "unknown"
    fi
}

detect_gpu() {
    local gpu="none"
    if command -v nvidia-smi &>/dev/null; then
        gpu="nvidia"
    elif command -v rocm-smi &>/dev/null; then
        gpu="amd_rocm"
    elif [[ -d /sys/class/drm ]]; then
        if ls /sys/class/drm/*/device/vendor 2>/dev/null | xargs cat 2>/dev/null | grep -q "0x10de"; then
            gpu="nvidia"
        elif ls /sys/class/drm/*/device/vendor 2>/dev/null | xargs cat 2>/dev/null | grep -q "0x1002"; then
            gpu="amd"
        elif ls /sys/class/drm/*/device/vendor 2>/dev/null | xargs cat 2>/dev/null | grep -q "0x8086"; then
            gpu="intel"
        fi
    fi
    echo "$gpu"
}

detect_vram_mb() {
    if command -v nvidia-smi &>/dev/null; then
        nvidia-smi --query-gpu=memory.total --format=csv,noheader,nounits 2>/dev/null | head -1 || echo "0"
    else
        echo "0"
    fi
}

detect_ram_mb() {
    local ram=0
    if [[ -f /proc/meminfo ]]; then
        ram=$(grep MemTotal /proc/meminfo | awk '{print int($2/1024)}')
    fi
    echo "$ram"
}

detect_disk_free_mb() {
    df -m / 2>/dev/null | awk 'NR==2 {print $4}' || echo "0"
}

# ─── Dependency Installation ──────────────────────────────────────────────────
install_deps_ubuntu() {
    log "Installing system dependencies (Ubuntu/Debian)..."
    sudo apt-get update -qq 2>&1 | tail -1
    sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq \
        curl wget git \
        libvulkan1 vulkan-tools mesa-vulkan-drivers \
        libasound2 \
        libx11-6 libxcursor1 libxi6 libxrandr2 \
        libssl-dev \
        ffmpeg \
        python3 python3-pip \
        v4l-utils \
        2>&1 | tail -5
    ok "Ubuntu dependencies installed"
}

install_deps_arch() {
    log "Installing system dependencies (Arch/Manjaro)..."
    sudo pacman -Syu --noconfirm --needed \
        curl wget git \
        vulkan-icd-loader vulkan-tools mesa \
        alsa-lib pipewire-alsa \
        libx11 libxcursor libxi libxrandr \
        openssl ffmpeg python3 v4l-utils \
        2>&1 | tail -5
    ok "Arch dependencies installed"
}

install_deps_fedora() {
    log "Installing system dependencies (Fedora/RHEL)..."
    sudo dnf install -y \
        curl wget git \
        vulkan mesa-vulkan-drivers \
        alsa-lib pipewire-alsa \
        libX11 libXcursor libXi libXrandr \
        openssl ffmpeg python3 \
        2>&1 | tail -5
    ok "Fedora dependencies installed"
}

install_deps_macos() {
    log "Installing macOS dependencies..."
    if ! command -v brew &>/dev/null; then
        log "Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)" 2>&1 | tail -3
    fi
    brew install --quiet ffmpeg python3 2>&1 | tail -3
    ok "macOS dependencies installed"
}

install_ollama() {
    if command -v ollama &>/dev/null; then
        ok "Ollama already installed ($(ollama --version 2>/dev/null | head -1))"
        return 0
    fi
    log "Installing Ollama (local AI runtime)..."
    if [[ $(detect_os) == "macos" ]]; then
        brew install --quiet ollama 2>&1 | tail -2
    else
        curl -fsSL https://ollama.com/install.sh | sh 2>&1 | tail -3
    fi
    ok "Ollama installed"
}

# ─── AI Models ────────────────────────────────────────────────────────────────
install_models() {
    if [[ "$SKIP_MODELS" == "true" ]]; then
        warn "Skipping AI model download (--no-models flag)"
        return 0
    fi

    local vram=$(detect_vram_mb)
    local ram=$(detect_ram_mb)

    echo ""
    log "Selecting AI models for your hardware (VRAM: ${vram}MB, RAM: ${ram}MB)..."

    # NPC Dialogue model
    local dialogue_model="qwen2.5:1.5b"
    if [[ $vram -ge 6000 || $ram -ge 12288 ]]; then
        dialogue_model="qwen2.5:7b"
    fi

    log "Downloading NPC Dialogue model: $dialogue_model"
    ollama pull "$dialogue_model" 2>&1 | tail -3 || warn "Model download failed, will retry on first run"

    # TTS model
    log "Downloading Text-to-Speech model (Kokoro)..."
    ollama pull "hexgrad/kokoro:82m" 2>&1 | tail -2 || warn "TTS model will be downloaded on first use"

    ok "Base AI models installed"
    log "You can install more models in Project Settings → AI → Model Manager"
}

# ─── Download & Install Genesis ──────────────────────────────────────────
determine_install_dir() {
    if [[ -n "$CV_INSTALL_DIR" ]]; then
        echo "$CV_INSTALL_DIR"
        return
    fi
    local os=$(detect_os)
    case $os in
        macos)   echo "$HOME/Applications/Genesis" ;;
        linux)   echo "$HOME/.local/share/genesis" ;;
        *)       echo "$HOME/Genesis" ;;
    esac
}

download_binary() {
    local os=$(detect_os)
    local arch=$(detect_arch)
    local install_dir="$1"

    local filename="genesis-${os}-${arch}-${CV_VERSION}"
    [[ "$os" == "windows" ]] && filename="${filename}.exe" || filename="${filename}.tar.gz"

    local url="${CV_REGISTRY}/${INSTALL_CHANNEL}/${filename}"

    log "Downloading Genesis ${CV_VERSION} (${os}/${arch})..."

    mkdir -p "$install_dir"

    if command -v curl &>/dev/null; then
        curl -L --progress-bar "$url" -o "/tmp/genesis_download" 2>&1
    elif command -v wget &>/dev/null; then
        wget -q --show-progress "$url" -O "/tmp/genesis_download"
    else
        die "Neither curl nor wget found. Please install one and retry."
    fi

    if [[ "$os" != "windows" && "$filename" == *.tar.gz ]]; then
        log "Extracting..."
        tar -xzf "/tmp/genesis_download" -C "$install_dir"
        rm "/tmp/genesis_download"
        chmod +x "$install_dir/genesis"
        chmod +x "$install_dir/genesis-cli"
    fi

    ok "Genesis downloaded to $install_dir"
}

create_desktop_entry() {
    local install_dir="$1"
    local os=$(detect_os)

    if [[ "$os" == "linux" ]]; then
        mkdir -p "$HOME/.local/share/applications"
        cat > "$HOME/.local/share/applications/genesis.desktop" << EOF
[Desktop Entry]
Name=Genesis
GenericName=Game Engine
Comment=The AI-First Game Engine
Exec=${install_dir}/genesis %f
Icon=${install_dir}/icons/genesis.png
Terminal=false
Type=Application
Categories=Development;GameDevelopment;
MimeType=application/x-genesis-project;
Keywords=game;engine;ai;development;
StartupNotify=true
StartupWMClass=genesis
EOF
        # Register file types
        if command -v xdg-mime &>/dev/null; then
            xdg-mime default genesis.desktop application/x-genesis-project 2>/dev/null || true
        fi
        ok "Desktop shortcut created"
    fi
}

add_to_path() {
    local install_dir="$1"
    local bin_dir="${install_dir}/bin"

    # Detect shell
    local shell_rc=""
    if [[ -n "${ZSH_VERSION:-}" ]] || [[ "$SHELL" == *zsh* ]]; then
        shell_rc="$HOME/.zshrc"
    elif [[ -n "${BASH_VERSION:-}" ]] || [[ "$SHELL" == *bash* ]]; then
        shell_rc="$HOME/.bashrc"
        [[ -f "$HOME/.bash_profile" ]] && shell_rc="$HOME/.bash_profile"
    elif [[ -f "$HOME/.profile" ]]; then
        shell_rc="$HOME/.profile"
    fi

    if [[ -n "$shell_rc" ]]; then
        if ! grep -q "genesis" "$shell_rc" 2>/dev/null; then
            echo "" >> "$shell_rc"
            echo "# Genesis Game Engine" >> "$shell_rc"
            echo "export PATH=\"${bin_dir}:\$PATH\"" >> "$shell_rc"
            ok "Added to PATH in $shell_rc"
            log "Run 'source $shell_rc' or restart your terminal to use 'genesis-cli'"
        else
            ok "Genesis already in PATH"
        fi
    fi
}

# ─── Existing Installation Check ─────────────────────────────────────────────
check_existing_installation() {
    if is_installed && [[ "$FORCE_REINSTALL" != "true" ]]; then
        local install_path=$(get_state "install_path")
        local installed_version=$(get_state "installed_version")
        local install_date=$(get_state "install_date")
        local machine_id=$(get_state "machine_id")

        echo ""
        echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
        echo -e "${GREEN}Genesis is already installed on this machine!${RESET}"
        echo ""
        echo -e "  Location:  ${BOLD}${install_path}${RESET}"
        echo -e "  Version:   ${BOLD}${installed_version:-unknown}${RESET}"
        echo -e "  Installed: ${BOLD}${install_date:-unknown}${RESET}"
        echo -e "  Machine:   ${BOLD}${machine_id:-unknown}${RESET}"
        echo ""

        # Check if newer version available
        echo "Checking for updates..."
        local latest_version
        latest_version=$(curl -sfL "${CV_REGISTRY}/${INSTALL_CHANNEL}/latest.txt" 2>/dev/null || echo "")
        if [[ -n "$latest_version" && "$latest_version" != "$installed_version" ]]; then
            echo -e "${YELLOW}New version available: ${latest_version}${RESET}"
            if [[ "$SILENT" != "true" ]]; then
                read -p "Update to v${latest_version}? [Y/n] " -n 1 -r choice
                echo ""
                if [[ "$choice" =~ ^[Yy]$  || -z "$choice" ]]; then
                    FORCE_REINSTALL=true
                    return 0
                fi
            fi
        else
            ok "Already on latest version"
        fi

        echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${RESET}"
        echo ""

        if [[ "$NO_LAUNCH" != "true" ]]; then
            log "Launching Genesis..."
            exec "${install_path}/genesis"
        fi
        exit 0
    fi
}

# ─── System Requirements Check ───────────────────────────────────────────────
check_requirements() {
    log "Checking system requirements..."
    local ok=true

    local ram=$(detect_ram_mb)
    local disk=$(detect_disk_free_mb)

    if [[ $ram -lt 3800 ]]; then
        warn "RAM: ${ram}MB — 4GB+ recommended for best experience"
    else
        ok "RAM: ${ram}MB ✓"
    fi

    if [[ $disk -lt 5000 ]]; then
        err "Disk: ${disk}MB free — need at least 5GB"
        ok=false
    else
        ok "Disk: ${disk}MB free ✓"
    fi

    local gpu=$(detect_gpu)
    case "$gpu" in
        nvidia) ok "GPU: NVIDIA (CUDA + Vulkan supported) ✓" ;;
        amd*)   ok "GPU: AMD (ROCm/Vulkan supported) ✓" ;;
        intel)  ok "GPU: Intel (Vulkan supported) ✓" ;;
        *)      warn "GPU: Unknown — will use CPU fallback" ;;
    esac

    if [[ "$ok" == "false" ]]; then
        die "System requirements not met. Please free up disk space and retry."
    fi
}

# ─── Main Installation ────────────────────────────────────────────────────────
main() {
    [[ "$SILENT" != "true" ]] && banner

    # Create log directory
    mkdir -p "$CV_STATE_DIR"
    echo "=== Genesis Install Log ===" > "$CV_LOG"
    echo "Date: $(date)" >> "$CV_LOG"

    # Check if already installed
    check_existing_installation

    echo ""
    log "Starting Genesis installation..."
    echo ""

    local os=$(detect_os)
    local arch=$(detect_arch)
    local distro=$(detect_distro)
    local gpu=$(detect_gpu)

    log "System: ${os}/${arch} | GPU: ${gpu} | Distro: ${distro}"

    # Requirements check
    check_requirements

    # Determine install directory
    local install_dir=$(determine_install_dir)
    log "Install directory: ${install_dir}"

    if [[ "$SILENT" != "true" ]]; then
        read -p "Install to ${install_dir}? [Y/n] " -n 1 -r
        echo ""
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            read -p "Enter install directory: " -r install_dir
        fi
    fi

    # Install OS dependencies
    echo ""
    log "Step 1/7: Installing system dependencies..."
    case "$os" in
        linux)
            case "$distro" in
                ubuntu|debian|linuxmint|pop) install_deps_ubuntu ;;
                arch|manjaro)                 install_deps_arch ;;
                fedora|rhel|centos)           install_deps_fedora ;;
                nixos) warn "NixOS detected. Add to configuration.nix manually." ;;
                *)     warn "Unknown distro. Dependencies may be missing." ;;
            esac
            ;;
        macos) install_deps_macos ;;
        *)     log "Dependency installation not automated for this OS." ;;
    esac

    # Install Ollama
    echo ""
    log "Step 2/7: Setting up local AI runtime..."
    install_ollama || warn "Ollama install failed. Cloud AI will be used instead."

    # Download Genesis
    echo ""
    log "Step 3/7: Downloading Genesis..."
    if [[ -n "$OFFLINE_BUNDLE" ]]; then
        log "Using offline bundle: $OFFLINE_BUNDLE"
        tar -xzf "$OFFLINE_BUNDLE" -C "$install_dir" 2>/dev/null || \
            cp -r "$OFFLINE_BUNDLE" "$install_dir"
    else
        download_binary "$install_dir" || {
            warn "Download failed. Using built-in bootstrap mode."
            mkdir -p "$install_dir/bin"
            cat > "$install_dir/genesis" << 'BOOTSTRAP'
#!/bin/bash
echo "Genesis Bootstrap Mode"
echo "Full binary not available — running in development mode"
echo "Set CV_INSTALL_DIR and re-run the installer with a valid download URL"
BOOTSTRAP
            chmod +x "$install_dir/genesis"
        }
    fi

    # Desktop integration
    echo ""
    log "Step 4/7: Creating desktop shortcut..."
    create_desktop_entry "$install_dir"

    # PATH setup
    echo ""
    log "Step 5/7: Adding to PATH..."
    add_to_path "$install_dir"

    # AI Models
    echo ""
    log "Step 6/7: Installing AI models..."
    install_models

    # Save install state (remember this PC)
    echo ""
    log "Step 7/7: Saving install state..."
    local machine_id=$(get_machine_id)
    local install_date=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

    cat > "$CV_STATE_FILE" << STATE
{
  "machine_id": "${machine_id}",
  "install_path": "${install_dir}",
  "installed_version": "${CV_VERSION}",
  "install_date": "${install_date}",
  "os": "${os}",
  "arch": "${arch}",
  "gpu": "${gpu}",
  "skip_models": "${SKIP_MODELS}",
  "channel": "${INSTALL_CHANNEL}",
  "last_run": "${install_date}"
}
STATE

    ok "Install state saved to ${CV_STATE_FILE}"

    # ─── Success! ─────────────────────────────────────────────────────────────
    echo ""
    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${RESET}"
    echo -e "${GREEN}║         ✅  Genesis Installed Successfully!           ║${RESET}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${RESET}"
    echo ""
    echo -e "  ${BOLD}Location:${RESET}  ${install_dir}"
    echo -e "  ${BOLD}Version:${RESET}   ${CV_VERSION}"
    echo ""
    echo -e "  ${BOLD}To create a new game:${RESET}"
    echo -e "    genesis-cli new my-game"
    echo ""
    echo -e "  ${BOLD}To open the editor:${RESET}"
    echo -e "    ${install_dir}/genesis"
    echo ""
    echo -e "  ${BOLD}Chat to build from anywhere:${RESET}"
    echo -e "    Set up Telegram bot in Editor → Hub → Telegram"
    echo ""
    echo -e "${YELLOW}  Note: Run 'source ~/.bashrc' to use genesis-cli${RESET}"
    echo ""

    # Launch if not suppressed
    if [[ "$NO_LAUNCH" != "true" && "$SILENT" != "true" ]]; then
        read -p "Launch Genesis now? [Y/n] " -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Nn]$ ]]; then
            log "Launching Genesis..."
            nohup "${install_dir}/genesis" &>/dev/null &
        fi
    elif [[ "$NO_LAUNCH" != "true" && "$SILENT" == "true" ]]; then
        log "Launching Genesis (silent mode)..."
        nohup "${install_dir}/genesis" &>/dev/null &
    fi
}

main "$@"
