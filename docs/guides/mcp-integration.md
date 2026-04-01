# MCP Server Guide

This document explains how to use the Model Context Protocol (MCP) server for AI agent integration with Anchor Engine.

## Overview

The MCP (Model Context Protocol) server enables AI agents to interact with Anchor Engine through standardized JSON-RPC 2.0 calls over stdio. This allows AI agents like Claude Desktop, Cursor, and Qwen Code to access your personal knowledge base.

## Starting the MCP Server

### Command Line
```bash
# Start MCP server with default settings
./target/release/anchor-mcp --db-path ./anchor.db

# Or use the start script
./start-mcp.bat
```

### Configuration Options
```bash
--db-path <PATH>    Path to SQLite database [default: ./anchor.db]
-v, --verbose       Enable verbose logging
-h, --help          Print help
-V, --version       Print version
```

## Available MCP Tools

### 1. anchor_query
Semantic search in your knowledge base.

**Method:** `anchor_query`
**Parameters:**
```json
{
  "query": "search query",
  "max_results": 10,
  "token_budget": 2048
}
```
**Returns:** Search results with content, provenance, and metadata.

### 2. anchor_distill
Run radial distillation to compress knowledge into decision records.

**Method:** `anchor_distill`
**Parameters:**
```json
{
  "seed": "optional seed query",
  "radius": 2,
  "output_format": "json"
}
```
**Returns:** Path to distillation output and statistics.

### 3. anchor_illuminate
BFS graph traversal to discover related concepts.

**Method:** `anchor_illuminate`
**Parameters:**
```json
{
  "seed": "starting query",
  "depth": 2,
  "max_nodes": 50
}
```
**Returns:** Related concepts with hop distances and gravity scores.

### 4. anchor_read_file
Read content from files by line ranges.

**Method:** `anchor_read_file`
**Parameters:**
```json
{
  "path": "/path/to/file",
  "start_line": 1,
  "end_line": 10
}
```
**Returns:** File content for the specified line range.

### 5. anchor_list_compounds
List available source files.

**Method:** `anchor_list_compounds`
**Parameters:**
```json
{
  "filter": "optional filter string"
}
```
**Returns:** List of available source files.

### 6. anchor_get_stats
Get system statistics.

**Method:** `anchor_get_stats`
**Parameters:** None
**Returns:** Database statistics (atoms, sources, tags, etc.).

### 7. anchor_ingest_text
Add raw text content (opt-in feature).

**Method:** `anchor_ingest_text`
**Parameters:**
```json
{
  "content": "text content",
  "filename": "filename.md",
  "bucket": "default"
}
```
**Returns:** Ingestion report.

### 8. anchor_ingest_file
Ingest files from filesystem (opt-in feature).

**Method:** `anchor_ingest_file`
**Parameters:**
```json
{
  "path": "/path/to/file"
}
```
**Returns:** Ingestion report.

## MCP Protocol Usage

### Example Request/Response

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "anchor_query",
  "params": {
    "query": "#rust programming",
    "max_results": 5,
    "token_budget": 2048
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "results": [
      {
        "id": 12345,
        "content": "Rust is a systems programming language...",
        "score": 0.85,
        "source": "rust_intro.md",
        "tags": ["#rust", "#programming", "#systems"],
        "provenance": "search"
      }
    ],
    "total": 1,
    "stats": {
      "query_time_ms": 45,
      "planets": 1,
      "moons": 0,
      "duration_ms": 45
    }
  }
}
```

## Integration with AI Agents

### Claude Desktop
Add to `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "anchor": {
      "command": "./target/release/anchor-mcp",
      "args": ["--db-path", "./anchor.db"]
    }
  }
}
```

### Qwen Code
Qwen Code automatically detects MCP servers. Just run:
```bash
./target/release/anchor-mcp --db-path ./anchor.db
```

Then use tools in chat:
```
/anchor_query query="What did we decide about authentication?"
```

### Cursor
Add to Cursor MCP settings:
```json
{
  "command": "./target/release/anchor-mcp",
  "args": ["--db-path", "./anchor.db"],
  "env": {
    "ANCHOR_API_URL": "http://localhost:3160"
  }
}
```

## Security Considerations

### Write Operations
- Write operations (`anchor_ingest_text`, `anchor_ingest_file`) are disabled by default
- Enable only when needed via configuration
- Use bucket isolation for untrusted content

### Rate Limiting
- Requests limited to 120 per minute by default
- Configurable via `rate_limit_requests_per_minute` setting

### API Keys
- Optional API key authentication available
- Configure via `require_api_key` setting

## Configuration

### MCP Server Settings
```json
{
  "mcp": {
    "enabled": true,
    "description": "Model Context Protocol server for AI agent memory",
    "security_note": "Only enable when needed. MCP exposes your knowledge graph to connected AI clients.",
    "allowed_clients": [],
    "require_api_key": false,
    "rate_limit_requests_per_minute": 120,
    "max_query_results": 100,
    "restrict_to_localhost": true,
    "allowed_operations": [
      "query",
      "read_file",
      "get_stats",
      "ingest",
      "distill",
      "illuminate",
      "list"
    ],
    "blocked_operations": []
  }
}
```

## Troubleshooting MCP Issues

### MCP Server Won't Start
- Verify database file exists and is accessible
- Check file permissions
- Ensure the database isn't locked by another process

### Tools Not Responding
- Verify the main anchor-engine server is running
- Check that the database path is correct
- Look for error messages in the logs

### Permission Issues
- Ensure MCP server has read/write access to the database
- Check that the mirrored_brain/ directory is accessible
- Verify file system permissions

## Performance Considerations

### Tool Response Times
- Most tools respond in <5ms
- Complex operations (distill, illuminate) may take longer
- Monitor system resources during heavy usage

### Concurrency
- MCP server handles requests sequentially
- Consider the impact of long-running operations
- Use appropriate timeouts in client applications

## Best Practices

1. **Secure Write Operations**: Keep write operations disabled unless specifically needed
2. **Rate Limiting**: Configure appropriate rate limits for your use case
3. **Monitoring**: Monitor tool usage and performance
4. **Isolation**: Use separate buckets for different content sources
5. **Backup**: Regularly backup your database and mirrored_brain/ directory

## Related Documentation

- [API Reference](../api/reference.md) - HTTP API for comparison
- [Architecture Spec](../../specs/current-standards/001-architecture-spec.md) - System architecture
- [Performance Guide](../technical/performance.md) - Performance optimization
- [Troubleshooting](common-issues.md) - General troubleshooting