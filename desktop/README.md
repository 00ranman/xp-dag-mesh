# XP DAG Mesh - Desktop Client

Cross-platform desktop application for running an XP DAG Mesh node. Built with [Tauri 2](https://tauri.app/) (Rust backend + web frontend).

## Architecture

The desktop client wraps the core mesh protocol into a native application:

```
desktop/
  src/                  # Frontend (TypeScript + Vite)
    index.html          # Dashboard UI
    main.ts             # Tauri IPC bridge
  src-tauri/            # Backend (Rust)
    src/
      main.rs           # Tauri commands + app entry
      node.rs           # Node lifecycle management
      wallet.rs         # Identity + quantitative finance tiered access
      validator.rs      # DAG validation with hybrid PoW/PoS
      sync.rs           # Mesh sync protocol + peer discovery
    Cargo.toml          # Rust dependencies
    tauri.conf.json     # Tauri configuration
    build.rs            # Build script
  package.json          # Node.js scripts
```

## Tiered Access Model

Nodes earn access tiers through quantitative finance metrics:

| Tier | Requirements | Capabilities |
|------|-------------|-------------|
| Observer | Default | Read-only mesh access |
| Contributor | Composite >= 0.30 | Propose DAG vertices |
| Validator | Composite >= 0.60 | Validate + sign blocks |
| Coordinator | Composite >= 0.85 | Full network coordination |

Composite score uses Kelly criterion (30%), Sharpe ratio (30%), uptime (20%), and validation accuracy (20%).

## Quick Start

### Prerequisites

- Node.js >= 18
- Rust >= 1.70
- System dependencies (see below)

### One-line install

```bash
git clone https://github.com/00ranman/xp-dag-mesh.git
cd xp-dag-mesh
chmod +x install.sh && ./install.sh
```

### Development

```bash
cd desktop
npm install
npm run tauri:dev
```

### Build for distribution

```bash
cd desktop
npm run tauri:build
```

Installers are output to `src-tauri/target/release/bundle/`:
- **macOS**: `.dmg` and `.app`
- **Windows**: `.msi` and `.exe`
- **Linux**: `.deb`, `.rpm`, and `.AppImage`

## System Dependencies

### macOS
No additional dependencies needed (Xcode Command Line Tools required).

### Linux (Debian/Ubuntu)
```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential libssl-dev libxdo-dev libayatana-appindicator3-dev librsvg2-dev
```

### Linux (Fedora)
```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel libxdo-devel libappindicator-gtk3-devel librsvg2-devel
```

## Tauri Commands

The backend exposes these IPC commands to the frontend:

| Command | Description |
|---------|------------|
| `generate_keypair` | Create Ed25519 node identity |
| `get_node_status` | Current node metrics |
| `start_mesh_node` | Start with role + bootstrap nodes |
| `stop_mesh_node` | Graceful shutdown |
| `get_xp_balance` | Query XP balance |
| `submit_entropy_claim` | Submit entropy reduction proof |
| `get_mesh_topology` | Connected peer graph |
| `get_tier_status` | Current tier + quant metrics |
