# Anchor UI

**Sovereign Knowledge Interface for Anchor Engine**

A lightweight, native desktop UI for the Anchor knowledge engine. Built with Tauri + Yew for a minimal, fast, and sovereign user experience.

## Status

🚧 **Under Construction** - Core structure complete, features being implemented.

## Architecture

- **Framework**: Tauri 2.0 (native desktop shell)
- **Frontend**: Yew 0.21 (Rust + WebAssembly)
- **Styling**: Custom CSS (glassmorphism theme)
- **HTTP Client**: Reqwest (WASM-compatible)

## Features (Planned/In Progress)

### P0 (Core)
- ✅ Multi-column search interface
- ✅ Chat with RAG context
- ⏳ Path manager (configure watched directories)
- ⏳ Quarantine page (review/cure atoms)
- ⏳ Settings panel

### P1 (Post-Launch)
- ⏳ Performance monitoring
- ⏳ Taxonomy editor
- ⏳ Graph visualization

## Building

### Prerequisites

1. **Rust** 1.75+ (stable)
2. **Tauri CLI**: `cargo install tauri-cli`

### Development

```bash
cd crates/anchor-ui
cargo tauri dev
```

### Production Build

```bash
cargo tauri build
```

Output will be in `target/release/`.

## Configuration

Edit `tauri.conf.json` to configure:
- Window size and title
- Bundle icons
- HTTP scope (default: `http://localhost:*`)

## API Integration

Anchor UI communicates with [anchor-engine](../anchor-engine) on `http://localhost:3160`.

Key endpoints:
- `GET /health` - Health check
- `GET /stats` - Database statistics
- `POST /v1/memory/search` - Search knowledge base
- `POST /v1/chat/completions` - Chat with context

## Project Structure

```
anchor-ui/
├── src/
│   ├── main.rs          # Tauri entry point
│   ├── lib.rs           # Yew app module exports
│   ├── app.rs           # Main app component + routing
│   ├── routes.rs        # Route definitions
│   ├── api.rs           # HTTP API client
│   ├── state.rs         # Global state management
│   ├── components/      # UI primitives
│   │   ├── button.rs
│   │   ├── input.rs
│   │   ├── glass_panel.rs
│   │   ├── badge.rs
│   │   └── loading.rs
│   └── features/        # Feature components
│       ├── search_column.rs
│       ├── chat_interface.rs
│       ├── path_manager.rs
│       ├── quarantine_page.rs
│       └── settings.rs
├── assets/
│   ├── style.css        # Base styles
│   └── components.css   # Component styles
├── index.html           # HTML entry point
├── tauri.conf.json      # Tauri configuration
└── Cargo.toml           # Rust dependencies
```

## Design Philosophy

1. **Sovereign** - No cloud dependencies, runs entirely locally
2. **Minimal** - Only essential features, no bloat
3. **Fast** - Native performance, no Electron overhead
4. **Beautiful** - Glassmorphism aesthetic, cyberpunk-inspired

## Color Scheme

```css
--bg-primary: #050507       /* Deep black */
--bg-secondary: #0a0a0f     /* Dark gray */
--accent-cyan: #06b6d4      /* Primary accent */
--accent-purple: #8b5cf6    /* Secondary accent */
--glass-bg: rgba(21, 21, 26, 0.6)  /* Glassmorphism */
```

## License

AGPL-3.0

## Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

**Part of Anchor OS v0 - Sovereign Knowledge Engine**
