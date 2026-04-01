# Installation Guide

This guide covers installing and setting up Anchor Engine Rust for different use cases.

## Prerequisites

- **Rust** (1.80+) - [Install Rust](https://rustup.rs/)
- **Git** (2.0+) - For cloning the repository
- **Windows/Linux/macOS** - Supported platforms

## Quick Installation

### Clone the Repository

```bash
git clone https://github.com/RSBalchII/anchor-engine-rust.git
cd anchor-engine-rust
```

### Build from Source

```bash
# Build release binaries
cargo build --release

# Binaries will be in target/release/
# - anchor-engine.exe (main server)
# - anchor-mcp.exe (MCP server)
```

### Run the Engine

```bash
# Start the main server
./target/release/anchor-engine --port 3160

# Or use the start script
./start.bat
```

## Configuration

### Database Path
```bash
# Specify custom database path
./target/release/anchor-engine --db-path ./custom/path/anchor.db
```

### Port Configuration
```bash
# Use different port
./target/release/anchor-engine --port 8080
```

### Verbose Logging
```bash
# Enable verbose logging
./target/release/anchor-engine --verbose
```

## MCP Server Setup

### Start MCP Server
```bash
# Start MCP server for AI agent integration
./target/release/anchor-mcp --db-path ./anchor.db
```

### MCP Configuration
```json
{
  "mcp": {
    "enabled": true,
    "allowed_clients": [],
    "require_api_key": false,
    "rate_limit_requests_per_minute": 120,
    "max_query_results": 100
  }
}
```

## Cross-Compilation (Optional)

For deploying to different architectures:

```bash
# Install target
rustup target add aarch64-unknown-linux-gnu

# Build for ARM64
cargo build --target aarch64-unknown-linux-gnu --release
```

## Docker Deployment (Coming Soon)

Docker support is planned for future releases.

## Troubleshooting

### Common Issues

1. **Build Errors**: Ensure Rust is updated to 1.80+
2. **Port Already in Use**: Change port with `--port` flag
3. **Permission Errors**: Run with appropriate file system permissions

### Performance Tips

- Use SSD storage for better I/O performance
- Allocate sufficient RAM for optimal performance
- Monitor memory usage during heavy ingestion

## Next Steps

- [API Reference](../api/reference.md) - Learn about available endpoints
- [Architecture Spec](../../specs/spec.md) - Understand the system design
- [Troubleshooting](../troubleshooting/common-issues.md) - Solve common problems