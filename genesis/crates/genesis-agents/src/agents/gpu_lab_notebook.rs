//! Genesis GPU Lab — Cloud GPU Integration
//!
//! For creators who don't have a local GPU:
//! - Connect to Kaggle, Google Colab, RunPod, Vast.ai, Lambda, Linode
//! - Generate ready-to-run notebooks with one click
//! - Auto-install all dependencies
//! - Browse and download HuggingFace models with search
//! - Auto-connect back to game engine (ngrok if no public IP)
//! - Password protection for the connection
//! - Get AI model suggestions based on your use case
//! - Run model generation jobs remotely
//! - Monitor GPU usage and costs
//!
//! The generated notebook:
//! 1. Installs Python deps (transformers, llama-cpp-python, etc.)
//! 2. Lets user browse/search HuggingFace models
//! 3. Downloads selected model
//! 4. Converts to GGUF if needed
//! 5. Starts connection server (WebSocket)
//! 6. Sets up ngrok tunnel (if no public IP)
//! 7. Sends connection URL back to game engine
//! 8. Game engine connects automatically

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// ─── Cloud Provider ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudGpuProvider {
    pub id: String,
    pub name: String,
    pub provider_type: ProviderType,
    pub connection: ConnectionConfig,
    pub gpus: Vec<GpuInstance>,
    pub status: ProviderStatus,
    pub monthly_budget_usd: Option<f32>,
    pub current_spend_usd: f32,
    pub preferred_for: Vec<String>, // task types
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderType {
    Kaggle    { username: String, key: String },
    Colab     { credentials_path: String },
    RunPod    { api_key: String },
    VastAi    { api_key: String },
    Lambda    { api_key: String },
    Linode    { api_key: String },
    Paperspace{ api_key: String },
    Custom    { endpoint: String, auth_token: String },
    Local,    // local GPU via Ollama
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: Option<String>,    // public IP if available
    pub port: u16,
    pub use_ngrok: bool,
    pub ngrok_token: Option<String>,
    pub password_hash: Option<String>,
    pub tunnel_url: Option<String>,  // set after connection established
    pub connected: bool,
    pub latency_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInstance {
    pub instance_id: String,
    pub gpu_model: String,
    pub vram_gb: f32,
    pub ram_gb: f32,
    pub running: bool,
    pub cost_per_hour_usd: f32,
    pub running_hours: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProviderStatus {
    Disconnected, Connecting, Connected, Running, Error(String),
}

// ─── HuggingFace Model Search ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HuggingFaceModel {
    pub model_id: String,         // "Qwen/Qwen2.5-7B-Instruct-GGUF"
    pub author: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub task_type: HfTaskType,
    pub downloads: u64,
    pub likes: u32,
    pub size_gb: f32,
    pub gated: bool,              // requires HF account/agreement
    pub versions: Vec<HfModelVersion>,
    pub recommended_for: Vec<String>, // Genesis task suggestions
    pub min_vram_gb: f32,
    pub can_run_cpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HfTaskType {
    TextGeneration,
    TextToImage,
    TextToAudio,
    TextTo3D,
    ImageToText,
    TextToSpeech,
    SpeechToText,
    Translation,
    Embedding,
    ObjectDetection,
    PoseEstimation,
    DepthEstimation,
    Inpainting,
    VideoGeneration,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfModelVersion {
    pub filename: String,
    pub size_gb: f32,
    pub quantization: String,   // "Q4_K_M", "F16", etc.
    pub recommended: bool,
}

// ─── Notebook Generator ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookConfig {
    pub provider: NotebookProvider,
    pub models_to_install: Vec<ModelInstallSpec>,
    pub engine_connection: EngineConnectionConfig,
    pub auto_start_server: bool,
    pub ngrok_enabled: bool,
    pub ngrok_token: Option<String>,
    pub password: Option<String>,
    pub install_extras: Vec<String>,
    pub gpu_acceleration: GpuAcceleration,
    pub auto_suggestions_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotebookProvider {
    Kaggle, Colab, JupyterLocal, Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInstallSpec {
    pub model_id: String,
    pub filename: Option<String>,
    pub quantization: Option<String>,
    pub task: String,
    pub backend: String, // "llama.cpp", "onnx", "ollama"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConnectionConfig {
    pub engine_public_ip: Option<String>,
    pub engine_port: u16,
    pub connection_token: String,
    pub fallback_to_ngrok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuAcceleration {
    Cuda, Metal, Rocm, Vulkan, Cpu,
}

pub struct GpuLabNotebookGenerator;

impl GpuLabNotebookGenerator {
    /// Generate a complete Jupyter notebook as a string
    pub fn generate(config: &NotebookConfig) -> String {
        let header = Self::header_cell(config);
        let install = Self::install_cell(config);
        let hf_search = Self::hf_search_cell();
        let model_suggestions = Self::model_suggestions_cell();
        let download = Self::download_models_cell(config);
        let convert = Self::convert_gguf_cell();
        let server = Self::start_server_cell(config);
        let ngrok = if config.ngrok_enabled { Self::ngrok_cell(config) } else { String::new() };
        let connect = Self::connect_to_engine_cell(config);
        let monitor = Self::monitor_cell();

        format!(r#"{{
 "nbformat": 4,
 "nbformat_minor": 5,
 "metadata": {{
   "kernelspec": {{ "display_name": "Python 3", "language": "python", "name": "python3" }},
   "accelerator": "GPU"
 }},
 "cells": [
   {header},
   {install},
   {hf_search},
   {model_suggestions},
   {download},
   {convert},
   {server},
   {ngrok},
   {connect},
   {monitor}
 ]
}}"#)
    }

    fn make_cell(cell_type: &str, source: &str, is_code: bool) -> String {
        let escaped = source.replace('"', "\\\"").replace('\n', "\\n");
        if is_code {
            format!(r#"{{
   "cell_type": "code",
   "source": "{escaped}",
   "metadata": {{}},
   "outputs": [],
   "execution_count": null
}}"#)
        } else {
            format!(r#"{{
   "cell_type": "markdown",
   "source": "{escaped}",
   "metadata": {{}}
}}"#)
        }
    }

    fn header_cell(config: &NotebookConfig) -> String {
        Self::make_cell("markdown", r#"# Genesis GPU Lab
**This notebook connects your cloud GPU to the Genesis game engine.**

## What this does:
1. Installs all required AI model dependencies
2. Lets you search and download models from HuggingFace
3. Starts a model server
4. Connects back to your game engine (auto-detects public IP or uses ngrok)

## Requirements:
- GPU runtime enabled (Runtime → Change runtime type → T4/A100)
- HuggingFace account (for gated models)
"#, false)
    }

    fn install_cell(config: &NotebookConfig) -> String {
        let gpu_flag = match &config.gpu_acceleration {
            GpuAcceleration::Cuda  => "--extra-index-url https://download.pytorch.org/whl/cu121",
            GpuAcceleration::Metal => "",
            _                      => "",
        };

        let code = format!(r#"
# ═══════════════════════════════════════════════
# STEP 1: Install dependencies
# ═══════════════════════════════════════════════
import subprocess, sys, os

def install(pkg):
    subprocess.check_call([sys.executable, '-m', 'pip', 'install', '-q', pkg])

print("Installing dependencies...")
install("huggingface_hub")
install("transformers")
install("accelerate")
install("llama-cpp-python {} --prefer-binary".format("--extra-index-url https://abetlen.github.io/llama-cpp-python/whl/cu121" if True else ""))
install("websockets")
install("fastapi")
install("uvicorn[standard]")
install("gguf")
install("pydantic")
install("httpx")
install("rich")  # pretty terminal output

# Optional: Ollama (easiest LLM server)
print("Checking for Ollama...")
os.system("curl -fsSL https://ollama.com/install.sh | sh")

print("✅ All dependencies installed!")
"#);
        Self::make_cell("code", &code, true)
    }

    fn hf_search_cell() -> String {
        let code = r#"
# ═══════════════════════════════════════════════
# STEP 2: Search HuggingFace for models
# ═══════════════════════════════════════════════
from huggingface_hub import HfApi, list_models
from rich.console import Console
from rich.table import Table

console = Console()
api = HfApi()

def search_models(query: str, task: str = None, limit: int = 20):
    """Search HuggingFace models"""
    models = api.list_models(
        search=query,
        task=task,
        sort="downloads",
        direction=-1,
        limit=limit,
        cardData=True,
    )

    table = Table(title=f"Search Results: '{query}'")
    table.add_column("Model ID", style="cyan", no_wrap=True)
    table.add_column("Downloads", justify="right", style="green")
    table.add_column("Size", justify="right")
    table.add_column("Tags")

    results = []
    for m in models:
        tags = ", ".join(m.tags[:3]) if m.tags else ""
        size_str = f"{m.safetensors.total / 1e9:.1f}GB" if hasattr(m, 'safetensors') and m.safetensors else "?"
        table.add_row(m.modelId, f"{m.downloads:,}", size_str, tags)
        results.append(m.modelId)

    console.print(table)
    return results

def list_model_files(model_id: str):
    """Show downloadable files for a model"""
    files = api.list_repo_files(model_id)
    gguf_files = [f for f in files if f.endswith('.gguf')]

    table = Table(title=f"Files in {model_id}")
    table.add_column("Filename", style="cyan")
    table.add_column("Type")

    for f in gguf_files:
        quant = "Q4_K_M" if "Q4_K_M" in f else "Q8" if "Q8" in f else "F16" if "F16" in f else "other"
        table.add_row(f, quant)

    console.print(table)
    return gguf_files

# Example searches:
# search_models("llama 3 instruct", task="text-generation")
# search_models("XTTS voice cloning")
# list_model_files("Qwen/Qwen2.5-7B-Instruct-GGUF")
print("🔍 Use search_models('your query') to find models")
print("📁 Use list_model_files('model/id') to see files")
"#;
        Self::make_cell("code", code, true)
    }

    fn model_suggestions_cell() -> String {
        let code = r#"
# ═══════════════════════════════════════════════
# STEP 3: Genesis Model Suggestions
# ═══════════════════════════════════════════════

CHRONOVERSE_MODELS = {
    "npc_dialogue_fast": {
        "model": "Qwen/Qwen2.5-1.5B-Instruct-GGUF",
        "file": "qwen2.5-1.5b-instruct-q4_k_m.gguf",
        "vram_gb": 1.5, "use_case": "Fast NPC dialogue, low latency"
    },
    "npc_dialogue_quality": {
        "model": "Qwen/Qwen2.5-7B-Instruct-GGUF",
        "file": "qwen2.5-7b-instruct-q4_k_m.gguf",
        "vram_gb": 5.0, "use_case": "High quality NPC dialogue"
    },
    "story_and_world": {
        "model": "bartowski/Mistral-7B-Instruct-v0.3-GGUF",
        "file": "Mistral-7B-Instruct-v0.3-Q4_K_M.gguf",
        "vram_gb": 5.0, "use_case": "Story generation, world building"
    },
    "code_generation": {
        "model": "Qwen/Qwen2.5-Coder-7B-Instruct-GGUF",
        "file": "qwen2.5-coder-7b-instruct-q4_k_m.gguf",
        "vram_gb": 5.0, "use_case": "Script generation, engine improvement"
    },
    "tts_fast": {
        "model": "hexgrad/Kokoro-82M",
        "file": None, "vram_gb": 0.5, "use_case": "Fast text-to-speech for NPCs"
    },
    "tts_quality": {
        "model": "coqui/XTTS-v2",
        "file": None, "vram_gb": 2.0, "use_case": "High quality, voice cloning capable TTS"
    },
    "vision": {
        "model": "Qwen/Qwen2.5-VL-7B-Instruct-GGUF",
        "file": "qwen2.5-vl-7b-instruct-q4_k_m.gguf",
        "vram_gb": 5.0, "use_case": "Visual QA, scene analysis"
    },
    "image_generation": {
        "model": "black-forest-labs/FLUX.1-schnell",
        "file": None, "vram_gb": 8.0, "use_case": "Generate textures, concept art"
    },
    "embedding": {
        "model": "nomic-ai/nomic-embed-text-v1.5-GGUF",
        "file": "nomic-embed-text-v1.5.Q8_0.gguf",
        "vram_gb": 0.3, "use_case": "Semantic memory search for agents"
    },
}

from rich.table import Table
from rich.console import Console
console = Console()

def show_suggestions(available_vram_gb: float = 8.0):
    table = Table(title=f"Recommended Models (VRAM: {available_vram_gb}GB)")
    table.add_column("Task", style="cyan")
    table.add_column("Model", style="green")
    table.add_column("VRAM", justify="right")
    table.add_column("Fits?", justify="center")
    table.add_column("Use Case")

    for task, info in CHRONOVERSE_MODELS.items():
        fits = "✅" if info["vram_gb"] <= available_vram_gb else "❌"
        table.add_row(task, info["model"].split("/")[-1], f"{info['vram_gb']}GB", fits, info["use_case"])

    console.print(table)

# Get available VRAM
import subprocess
result = subprocess.run(['nvidia-smi', '--query-gpu=memory.total', '--format=csv,noheader,nounits'],
                       capture_output=True, text=True)
vram = float(result.stdout.strip()) / 1024 if result.returncode == 0 else 8.0
show_suggestions(vram)
print(f"\nDetected GPU VRAM: {vram:.1f}GB")

# Select models to install
SELECTED_MODELS = ["npc_dialogue_quality", "tts_fast", "embedding"]
print(f"\nSelected for installation: {SELECTED_MODELS}")
print("Modify SELECTED_MODELS list above to change selection")
"#;
        Self::make_cell("code", code, true)
    }

    fn download_models_cell(config: &NotebookConfig) -> String {
        let code = r#"
# ═══════════════════════════════════════════════
# STEP 4: Download Selected Models
# ═══════════════════════════════════════════════
from huggingface_hub import snapshot_download, hf_hub_download
import os

MODELS_DIR = "/kaggle/working/models" if os.path.exists("/kaggle") else "/content/models"
os.makedirs(MODELS_DIR, exist_ok=True)

downloaded_paths = {}

for task_name in SELECTED_MODELS:
    info = CHRONOVERSE_MODELS[task_name]
    model_id = info["model"]
    filename = info.get("file")

    print(f"\n📥 Downloading: {model_id}")

    try:
        if filename:
            # Download specific GGUF file
            path = hf_hub_download(
                repo_id=model_id,
                filename=filename,
                local_dir=MODELS_DIR,
            )
        else:
            # Download full model
            path = snapshot_download(
                repo_id=model_id,
                local_dir=os.path.join(MODELS_DIR, model_id.replace("/", "_")),
            )

        downloaded_paths[task_name] = path
        size = os.path.getsize(path) / 1e9 if os.path.isfile(path) else 0
        print(f"   ✅ Downloaded: {path} ({size:.2f}GB)")
    except Exception as e:
        print(f"   ❌ Failed: {e}")
        print(f"   Try: search_models('{model_id}')")

# Also: custom model download
def download_custom(model_id: str, filename: str = None):
    """Download any model by ID"""
    if filename:
        return hf_hub_download(repo_id=model_id, filename=filename, local_dir=MODELS_DIR)
    return snapshot_download(repo_id=model_id, local_dir=os.path.join(MODELS_DIR, model_id.replace("/","_")))

print(f"\n✅ Downloaded {len(downloaded_paths)} models to {MODELS_DIR}")
"#;
        Self::make_cell("code", code, true)
    }

    fn convert_gguf_cell() -> String {
        let code = r#"
# ═══════════════════════════════════════════════
# STEP 5: Convert to GGUF (if needed)
# ═══════════════════════════════════════════════
import subprocess, os

def convert_to_gguf(model_path: str, quantization: str = "Q4_K_M"):
    """Convert a HuggingFace model to GGUF format"""
    if model_path.endswith('.gguf'):
        print(f"Already GGUF: {model_path}")
        return model_path

    # Install llama.cpp convert script
    if not os.path.exists("/content/llama.cpp"):
        subprocess.run(["git", "clone", "--depth=1", "https://github.com/ggerganov/llama.cpp", "/content/llama.cpp"])
        subprocess.run(["pip", "install", "-q", "-r", "/content/llama.cpp/requirements.txt"])

    output = model_path + f".{quantization}.gguf"
    cmd = [
        "python", "/content/llama.cpp/convert_hf_to_gguf.py",
        model_path, "--outfile", output, "--outtype", quantization.lower()
    ]

    print(f"Converting {model_path} → {output}")
    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode == 0:
        print(f"✅ Converted: {output}")
        return output
    else:
        print(f"❌ Conversion failed: {result.stderr}")
        return model_path

print("Conversion tool ready. Use convert_to_gguf(path) to convert models.")
"#;
        Self::make_cell("code", code, true)
    }

    fn start_server_cell(config: &NotebookConfig) -> String {
        let port = config.engine_connection.engine_port;
        let token = &config.engine_connection.connection_token;
        let password = config.engine_connection.engine_port.to_string();

        let code = format!(r#"
# ═══════════════════════════════════════════════
# STEP 6: Start Model Server
# ═══════════════════════════════════════════════
import asyncio, json, os, threading
import websockets
from llama_cpp import Llama

# Load models
loaded_models = {{}}

for task_name, path in downloaded_paths.items():
    if path.endswith('.gguf'):
        print(f"Loading {{task_name}}...")
        try:
            loaded_models[task_name] = Llama(
                model_path=path,
                n_ctx=4096,
                n_gpu_layers=-1,  # Use all GPU layers
                verbose=False,
            )
            print(f"  ✅ {{task_name}} loaded")
        except Exception as e:
            print(f"  ❌ {{task_name}} failed: {{e}}")

# Start Ollama for easy management
os.system("ollama serve &")

CONNECTION_TOKEN = "{token}"
SERVER_PORT = {port}

async def handle_connection(websocket, path):
    """Handle incoming connection from Genesis engine"""
    # Authenticate
    auth_msg = await websocket.recv()
    auth = json.loads(auth_msg)

    if auth.get("token") != CONNECTION_TOKEN:
        await websocket.send(json.dumps({{"error": "Invalid token"}}))
        await websocket.close()
        return

    print(f"✅ Genesis engine connected!")
    await websocket.send(json.dumps({{
        "status": "connected",
        "models": list(loaded_models.keys()),
        "capabilities": ["text_generation", "embedding", "tts"],
    }}))

    async for message in websocket:
        try:
            request = json.loads(message)
            task = request.get("task")
            model_name = request.get("model", list(loaded_models.keys())[0] if loaded_models else None)

            if task == "generate" and model_name in loaded_models:
                model = loaded_models[model_name]
                result = model(
                    request.get("prompt", ""),
                    max_tokens=request.get("max_tokens", 256),
                    temperature=request.get("temperature", 0.7),
                    stop=request.get("stop", []),
                )
                response = {{"result": result["choices"][0]["text"], "task": task}}
            elif task == "ping":
                response = {{"pong": True}}
            else:
                response = {{"error": f"Unknown task: {{task}}"}}

            await websocket.send(json.dumps(response))
        except Exception as e:
            await websocket.send(json.dumps({{"error": str(e)}}))

# Start server in background
def run_server():
    loop = asyncio.new_event_loop()
    asyncio.set_event_loop(loop)
    start_server = websockets.serve(handle_connection, "0.0.0.0", SERVER_PORT)
    loop.run_until_complete(start_server)
    print(f"🚀 GPU Lab server running on port {{SERVER_PORT}}")
    loop.run_forever()

server_thread = threading.Thread(target=run_server, daemon=True)
server_thread.start()

import time; time.sleep(1)
print(f"Server started! Models available: {{list(loaded_models.keys())}}")
"#);
        Self::make_cell("code", &code, true)
    }

    fn ngrok_cell(config: &NotebookConfig) -> String {
        let token = config.ngrok_token.as_deref().unwrap_or("YOUR_NGROK_TOKEN_HERE");
        let port = config.engine_connection.engine_port;

        let code = format!(r#"
# ═══════════════════════════════════════════════
# STEP 7: Setup ngrok Tunnel (if no public IP)
# ═══════════════════════════════════════════════
# ngrok creates a public URL that forwards to your server
# Get free token at: https://dashboard.ngrok.com

!pip install -q pyngrok

from pyngrok import ngrok, conf

NGROK_TOKEN = "{token}"

if NGROK_TOKEN and NGROK_TOKEN != "YOUR_NGROK_TOKEN_HERE":
    conf.get_default().auth_token = NGROK_TOKEN
    tunnel = ngrok.connect({port}, "tcp")
    public_url = tunnel.public_url

    # Parse host and port from ngrok URL
    # Format: tcp://0.tcp.ngrok.io:PORT
    parts = public_url.replace("tcp://", "").split(":")
    ngrok_host = parts[0]
    ngrok_port = int(parts[1]) if len(parts) > 1 else {port}

    print(f"🌐 ngrok tunnel active!")
    print(f"   Public URL: {{public_url}}")
    print(f"   Host: {{ngrok_host}}")
    print(f"   Port: {{ngrok_port}}")

    CONNECTION_INFO = {{
        "host": ngrok_host,
        "port": ngrok_port,
        "token": CONNECTION_TOKEN,
        "url": public_url,
    }}
else:
    # Try to get public IP
    import httpx
    try:
        ip = httpx.get("https://api.ipify.org").text
        print(f"🌐 Public IP: {{ip}}")
        CONNECTION_INFO = {{
            "host": ip, "port": {port}, "token": CONNECTION_TOKEN,
            "url": f"ws://{{ip}}:{port}",
        }}
    except:
        print("❌ Could not determine public IP. Add ngrok token.")
        CONNECTION_INFO = {{"error": "No public access"}}

print(f"\n📋 Connection info:\n{{json.dumps(CONNECTION_INFO, indent=2)}}")
"#);
        Self::make_cell("code", &code, true)
    }

    fn connect_to_engine_cell(config: &NotebookConfig) -> String {
        let code = r#"
# ═══════════════════════════════════════════════
# STEP 8: Connect to Genesis Engine
# ═══════════════════════════════════════════════
# The engine will auto-detect this server if on same network
# Or paste the connection info into the engine's GPU Lab settings

print("=" * 60)
print("CHRONOVERSE GPU LAB READY")
print("=" * 60)
print(f"\nConnection Token: {CONNECTION_TOKEN}")
print(f"Connection Info:")
import json
print(json.dumps(CONNECTION_INFO, indent=2))
print("\n")
print("In Genesis Editor:")
print("  1. Open Project Settings → GPU Lab")
print("  2. Enter the Connection Token above")
print("  3. Enter the Host and Port above")
print("  4. Click 'Connect'")
print("\nOr copy this into game project settings:")
print(json.dumps(CONNECTION_INFO))
print("=" * 60)

# Alternatively: send connection info to engine automatically
import httpx
engine_url = "http://YOUR_ENGINE_IP:8080/api/v1/gpu_lab/register"
try:
    r = httpx.post(engine_url, json=CONNECTION_INFO, timeout=5.0)
    if r.status_code == 200:
        print("✅ Auto-registered with Genesis engine!")
    else:
        print(f"Manual registration needed (engine returned {r.status_code})")
except:
    print("Auto-registration skipped (engine not reachable from here)")
    print("Use the connection info above to connect manually.")
"#;
        Self::make_cell("code", code, true)
    }

    fn monitor_cell() -> String {
        let code = r#"
# ═══════════════════════════════════════════════
# STEP 9: Monitor (run this to keep session alive)
# ═══════════════════════════════════════════════
import time, subprocess, os

def get_gpu_info():
    try:
        result = subprocess.run(
            ['nvidia-smi', '--query-gpu=name,memory.used,memory.total,temperature.gpu,utilization.gpu',
             '--format=csv,noheader,nounits'],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            parts = result.stdout.strip().split(', ')
            return {
                "name": parts[0], "vram_used_mb": int(parts[1]),
                "vram_total_mb": int(parts[2]), "temp_c": int(parts[3]), "util_pct": int(parts[4])
            }
    except: pass
    return {}

request_count = 0
start_time = time.time()

print("🔄 Monitoring active. GPU Lab is running.")
print("   This cell keeps the session alive.")
print("   Press Stop (■) to shut down the GPU Lab.")

try:
    while True:
        gpu = get_gpu_info()
        uptime = int(time.time() - start_time)

        if gpu:
            vram_pct = gpu['vram_used_mb'] / gpu['vram_total_mb'] * 100
            print(f"\r⚡ [{uptime}s] {gpu['name']} | "
                  f"VRAM: {gpu['vram_used_mb']}MB/{gpu['vram_total_mb']}MB ({vram_pct:.0f}%) | "
                  f"Temp: {gpu['temp_c']}°C | "
                  f"GPU: {gpu['util_pct']}%", end="", flush=True)
        else:
            print(f"\r⚡ [{uptime}s] Running... | Models: {list(loaded_models.keys())}", end="", flush=True)

        time.sleep(30)
except KeyboardInterrupt:
    print("\n\nGPU Lab stopped.")
"#;
        Self::make_cell("code", code, true)
    }
}

/// Notebook export targets
pub struct NotebookExporter;

impl NotebookExporter {
    /// Generate notebook and return as JSON string
    pub fn generate_for_kaggle(engine_ip: Option<&str>, port: u16, token: &str) -> String {
        let config = NotebookConfig {
            provider: NotebookProvider::Kaggle,
            models_to_install: vec![
                ModelInstallSpec {
                    model_id: "Qwen/Qwen2.5-7B-Instruct-GGUF".to_string(),
                    filename: Some("qwen2.5-7b-instruct-q4_k_m.gguf".to_string()),
                    quantization: Some("Q4_K_M".to_string()),
                    task: "npc_dialogue_quality".to_string(),
                    backend: "llama.cpp".to_string(),
                },
            ],
            engine_connection: EngineConnectionConfig {
                engine_public_ip: engine_ip.map(|s| s.to_string()),
                engine_port: port,
                connection_token: token.to_string(),
                fallback_to_ngrok: true,
            },
            auto_start_server: true,
            ngrok_enabled: engine_ip.is_none(),
            ngrok_token: None,
            password: None,
            install_extras: Vec::new(),
            gpu_acceleration: GpuAcceleration::Cuda,
            auto_suggestions_enabled: true,
        };

        GpuLabNotebookGenerator::generate(&config)
    }
}
