# GENESIS Engine
### *Where Worlds Begin*

> The AI-First Game Engine — build entire games by describing them.
> 32 AI agents. Local LLMs. 30+ cloud providers. Full 3D studio.

## Quick Install

```bash
curl -fsSL https://genesis-engine.io/install.sh | bash
```

```bash
genesis                            # open editor
genesis-cli new my_game --ai       # new project with AI planning
genesis-cli run                    # run game
genesis-cli build --target web     # export to web
genesis-cli doctor                 # system check
```

## Stats

| Metric | Value |
|--------|-------|
| Crates | 56 |
| Rust source files | 165 |
| Lines of code | 32,000+ |
| AI Agents | 32 |
| AI Providers | 30+ |
| Creature archetypes | 200+ |
| Material types | 20+ |
| Biome types | 30+ |
| VFX compositor nodes | 80+ |
| Camera modes | 40+ |

## Features

- **AI Council** — 32 agents: world gen, NPC brains, story, combat, music, VFX, lighting, modeler
- **Local LLM** — auto hardware-adaptive (TinyLlama 1.1B → Llama 3.3 70B based on your specs)
- **30+ AI Providers** — Anthropic, OpenAI, Groq, DeepSeek, Mistral, Together, Fireworks + more
- **Rollback Netcode** — GGPO-style, 8-frame window, ML input prediction
- **Open World** — 256m chunks, async streaming, no loading screens
- **Destruction** — Voronoi fracturing, structural integrity, progressive damage
- **Fluid Sim** — SPH/DFSPH (100k+ particles), water/lava/honey/blood/acid
- **Virtual Studio** — Blender-equivalent modeler + Unreal virtual production + 80-node VFX compositor + DAW
- **Creatures** — 200+ species, genetics, taming, riding, pack AI, migration
- **ML Difficulty** — player behavioral modeling, invisible adaptive difficulty
- **Anti-Cheat** — behavioral analysis, teleport/speed/aimbot detection, soft/hard bans
- **Social** — always-on replay buffer, highlight detection, share to TikTok/YouTube/Twitch
- **Marketplace** — asset store, plugin store, game store (88% developer revenue)
- **Universal Install** — `curl | bash` on Linux/macOS/Windows

## CLI

```bash
# AI from terminal
genesis-cli ai ask "10 dark fantasy tavern names"
genesis-cli ai npc "grumpy dwarven blacksmith with a secret"
genesis-cli ai scene "foggy graveyard at midnight"
genesis-cli ai quest "find the missing merchant caravan"
genesis-cli ai script "double jump with coyote time"
genesis-cli ai gdd "soulslike set in eternal night" --pages 10

# Project
genesis-cli new my_game --template rpg --ai
genesis-cli build --target web --release
genesis-cli stats
genesis-cli validate

# Server
genesis-server --port 7070 --slots 32 --anti-cheat
```

## Compared to Unity/Unreal/Godot

| Feature | Unity | Unreal | Godot | GENESIS |
|---------|-------|--------|-------|---------|
| AI Agents | ✗ | ✗ | ✗ | 32 built-in |
| Local LLM | ✗ | ✗ | ✗ | Auto-adaptive |
| 1-command install | ✗ | ✗ | ✗ | curl \| bash |
| Rollback netcode | Manual | Manual | ✗ | Built-in |
| Voronoi destruction | Store | Chaos | ✗ | Built-in |
| SPH fluid | Store | Niagara | ✗ | Built-in |
| Creature ecosystem | ✗ | ✗ | ✗ | 200+ species |
| Full 3D studio | ✗ | Partial | ✗ | Blender-level |
| Social recording | ✗ | ✗ | ✗ | Built-in |
| Revenue share | 70% | 88% | N/A | 88% |

## Architecture

Built in Rust with: WGPU (renderer), Rapier3D (physics), tokio (async), hecs (ECS), mlua (Lua 5.4), rhai (scripting), egui (UI).

*GENESIS Engine — https://genesis-engine.io*
