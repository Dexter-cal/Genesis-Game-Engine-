# Genesis Engine: Technical Documentation

## Overview
Genesis is a high-performance, AI-first game engine built in Rust. It utilizes a massive 56-crate modular architecture to provide a unified workflow from 3D modeling and rigging to real-time simulation and network deployment.

## Core Architecture

### 1. ECS & Core (The Backbone)
- **`genesis-ecs`**: A high-performance Entity Component System using `hecs`. It manages all game state and entities.
- **`genesis-core`**: Defines base traits and systems for the engine's lifecycle.
- **`genesis-math`**: A performance-tuned math library based on `glam`.

### 2. The AI Stack (Intelligence First)
- **`genesis-agents`**: High-level agent orchestration. Includes vision, terrain analysis, and world-awareness systems.
- **`genesis-ai`**: Low-level AI utilities, including the **AI Rigging Solver** which automatically generates skeletal hierarchies from meshes.
- **`genesis-ml`**: Integration for machine learning models (PyTorch/TensorFlow bindings) for real-time behavior training.

### 3. Professional Creative Suite
- **Genesis Engine (IDE)**: A unified environment for scene building. Includes:
    - **Scene Hierarchy**: Real-time entity management.
    - **Inspector**: Deep-linking to ECS components.
    - **AI Assistant**: A natural language interface for scene manipulation (e.g., "Add 10 trees with physics").
- **Genesis Studio**: A specialized 3D authoring tool.
    - **Rigging Tree**: Advanced skeletal management.
    - **Modifier Stack**: Procedural modeling and deformation.
    - **Animation Timeline**: Frame-perfect control over state-based animations.

## Development Workflow

### Building the Workspace
```bash
cargo build --workspace
```

### Running the Engine
```bash
cargo run -p genesis-app
```

### AI-Driven Content Creation
1. **Model**: Import or generate a mesh in Genesis Studio.
2. **Auto-Rig**: Use the `genesis-ai` rigging solver to generate a skeleton.
3. **Behavior**: Attach a `GenesisAgent` component via the Engine's Inspector.
4. **Deploy**: Use `genesis-export` to package for Windows, Linux, or Console.

## Advanced Features
- **Network Prediction**: `genesis-network` provides built-in rollback and client-side prediction.
- **VFX Compositor**: `genesis-vfx-compositor` allows node-based particle and post-processing design.
- **Advanced Physics**: Multi-threaded physics integration via `genesis-physics-advanced`.

---
© 2024 Genesis Engine Team. "Where Worlds Begin."
