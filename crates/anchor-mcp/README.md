# Anchor MCP Server

**MCP (Model Context Protocol) server for Anchor Engine** - exposes knowledge graph tools via JSON-RPC 2.0 over stdio.

## Overview

This crate provides a bridge between AI agents and the Anchor Engine knowledge graph. It implements the Model Context Protocol (MCP) specification, allowing agents to:

- Query the knowledge graph using the STAR algorithm
- Ingest new content (text or files)
- Retrieve file contents
- List compounds/molecules
- Get database statistics

## Installation

```bash
# Build the MCP server
cargo build --release

# Binary location
./target/release/anchor-mcp
```

## Usage

### Starting the Server

```bash
# With default database path (./anchor.db)
./target/release/anchor-mcp

# With custom database path
./target/release/anchor-mcp --db-path /path/to/anchor.db

# With verbose logging
./target/release/anchor-mcp --db-path ./anchor.db -v
```

### Command-Line Options

| Option | Description | Default |
|--------|-------------|---------|
| `--db-path` | Path to SQLite database | `./anchor.db` |
| `-v, --verbose` | Enable debug logging | `false` |
| `-h, --help` | Show help message | - |
| `-V, --version` | Show version | - |

## Protocol

The server communicates via **JSON-RPC 2.0** over **stdio**.

### Request Format

```json
{
  "jsonrpc": "2.0",
  "id": <any>,
  "method": "<method_name>",
  "params": { ... }
}
```

### Response Format

**Success:**
```json
{
  "jsonrpc": "2.0",
  "id": <same as request>,
  "result": { ... }
}
```

**Error:**
```json
{
  "jsonrpc": "2.0",
  "id": <same as request>,
  "error": {
    "code": <error_code>,
    "message": "<error_message>"
  }
}
```

### Error Codes

| Code | Name | Description |
|------|------|-------------|
| -32700 | PARSE_ERROR | Invalid JSON |
| -32600 | INVALID_REQUEST | Invalid JSON-RPC format |
| -32601 | METHOD_NOT_FOUND | Method doesn't exist |
| -32602 | INVALID_PARAMS | Invalid method parameters |
| -32603 | INTERNAL_ERROR | Internal server error |
| -32000 | APPLICATION_ERROR | Application-specific error |

## Available Tools

### `anchor_query`

Search the knowledge graph using the STAR algorithm.

**Parameters:**
- `query` (string, required): Search query (e.g., "#rust" or "OAuth setup")
- `max_results` (integer, optional): Maximum results to return (default: 50)

**Example Request:**
```json
{"jsonrpc":"2.0","id":1,"method":"anchor_query","params":{"query":"#rust","max_results":10}}
```

**Example Response:**
```json
{
  "jsonrpc":"2.0",
  "id":1,
  "result":{
    "results":[
      {
        "id": 42,
        "content": "Rust is a systems programming language...",
        "score": 0.85,
        "source": "docs/rust.md",
        "tags": ["#rust", "#programming"],
        "provenance": "tag_match"
      }
    ],
    "total": 15,
    "stats": {
      "query_time_ms": 45,
      "planets": 5,
      "moons": 10
    }
  }
}
```

### `anchor_distill`

Perform knowledge distillation (semantic compression).

**Parameters:**
- `seed` (string, optional): Seed query for distillation
- `radius` (integer, optional): Graph traversal radius (default: 2)

**Example Request:**
```json
{"jsonrpc":"2.0","id":2,"method":"anchor_distill","params":{"seed":"#rust","radius":3}}
```

**Example Response:**
```json
{
  "jsonrpc":"2.0",
  "id":2,
  "result":{
    "output_path": "/path/to/distilled_output.json",
    "compression_ratio": 0.15,
    "lines_unique": 1250,
    "duration_ms": 3420
  }
}
```

### `anchor_illuminate`

Perform BFS graph traversal from a seed.

**Parameters:**
- `seed` (string, required): Seed atom/molecule ID
- `depth` (integer, optional): Traversal depth (default: 2)

**Example Request:**
```json
{"jsonrpc":"2.0","id":3,"method":"anchor_illuminate","params":{"seed":"atom:42","depth":3}}
```

**Example Response:**
```json
{
  "jsonrpc":"2.0",
  "id":3,
  "result":{
    "nodes": [...],
    "edges": [...]
  }
}
```

### `anchor_read_file`

Read content from a file.

**Parameters:**
- `path` (string, required): Absolute file path
- `start_line` (integer, optional): Starting line number (0-indexed)
- `end_line` (integer, optional): Ending line number (exclusive)

**Example Request:**
```json
{"jsonrpc":"2.0","id":4,"method":"anchor_read_file","params":{"path":"/path/to/file.md","start_line":0,"end_line":10}}
```

**Example Response:**
```json
{
  "jsonrpc":"2.0",
  "id":4,
  "result":{
    "path": "/path/to/file.md",
    "content": "File content here..."
  }
}
```

### `anchor_list_compounds`

List all compounds (molecules) in the database.

**Parameters:**
- `filter` (string, optional): Filter by tag or source

**Example Request:**
```json
{"jsonrpc":"2.0","id":5,"method":"anchor_list_compounds","params":{"filter":"#rust"}}
```

**Example Response:**
```json
{
  "jsonrpc":"2.0",
  "id":5,
  "result":{
    "compounds": [...],
    "total": 150
  }
}
```

### `anchor_get_stats`

Get database statistics.

**Parameters:** None

**Example Request:**
```json
{"jsonrpc":"2.0","id":6,"method":"anchor_get_stats","params":{}}
```

**Example Response:**
```json
{
  "jsonrpc":"2.0",
  "id":6,
  "result":{
    "atoms": 150000,
    "molecules": 28000,
    "sources": 436,
    "tags": 5200
  }
}
```

### `anchor_ingest_text`

Ingest raw text content into the knowledge graph.

**Parameters:**
- `content` (string, required): Text content to ingest
- `filename` (string, required): Source filename (for provenance)
- `bucket` (string, optional): Bucket/category (default: "default")

**Example Request:**
```json
{"jsonrpc":"2.0","id":7,"method":"anchor_ingest_text","params":{"content":"Rust is safe and fast.","filename":"notes.md","bucket":"personal"}}
```

**Example Response:**
```json
{
  "jsonrpc":"2.0",
  "id":7,
  "result":{
    "success": true,
    "atoms_ingested": 5,
    "molecules_created": 2,
    "duration_ms": 125
  }
}
```

### `anchor_ingest_file`

Ingest content from a file.

**Parameters:**
- `path` (string, required): Absolute file path
- `bucket` (string, optional): Bucket/category (default: "default")

**Example Request:**
```json
{"jsonrpc":"2.0","id":8,"method":"anchor_ingest_file","params":{"path":"/path/to/document.md","bucket":"docs"}}
```

**Example Response:**
```json
{
  "jsonrpc":"2.0",
  "id":8,
  "result":{
    "success": true,
    "atoms_ingested": 42,
    "molecules_created": 15,
    "duration_ms": 890
  }
}
```

## Integration Examples

### Python

```python
import subprocess
import json

# Start MCP server
process = subprocess.Popen(
    ["./target/release/anchor-mcp", "--db-path", "./anchor.db"],
    stdin=subprocess.PIPE,
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    text=True
)

def send_request(method, params):
    request = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    }
    process.stdin.write(json.dumps(request) + "\n")
    process.stdin.flush()
    response = json.loads(process.stdout.readline())
    return response

# Query the knowledge graph
result = send_request("anchor_query", {"query": "#rust", "max_results": 10})
print(result["result"]["results"])
```

### Node.js

```javascript
const { spawn } = require('child_process');

// Start MCP server
const mcp = spawn('./target/release/anchor-mcp', ['--db-path', './anchor.db']);

function sendRequest(method, params) {
  return new Promise((resolve) => {
    const request = {
      jsonrpc: '2.0',
      id: 1,
      method,
      params
    };
    
    mcp.stdout.once('data', (data) => {
      resolve(JSON.parse(data.toString()));
    });
    
    mcp.stdin.write(JSON.stringify(request) + '\n');
  });
}

// Query the knowledge graph
sendRequest('anchor_query', { query: '#rust', max_results: 10 })
  .then(result => console.log(result.result));
```

## Testing

```bash
# Run integration tests
cargo test --package anchor-mcp

# Run with verbose output
cargo test --package anchor-mcp -- --nocapture
```

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    AI Agent / Client                     │
└────────────────────┬────────────────────────────────────┘
                     │ JSON-RPC 2.0 (stdio)
                     ▼
┌─────────────────────────────────────────────────────────┐
│                   Anchor MCP Server                      │
│  ┌─────────────────────────────────────────────────┐    │
│  │  Request Handler  │  Response Builder           │    │
│  └─────────────────────────────────────────────────┘    │
│                     │                                    │
│  ┌─────────────────────────────────────────────────┐    │
│  │           AnchorService (anchor-engine)         │    │
│  │  - search()                                     │    │
│  │  - ingest()                                     │    │
│  │  - distill()                                    │    │
│  │  - illuminate()                                 │    │
│  │  - read_file()                                  │    │
│  └─────────────────────────────────────────────────┘    │
│                     │                                    │
│  ┌─────────────────────────────────────────────────┐    │
│  │              SQLite Database                     │    │
│  └─────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

## Troubleshooting

### Server won't start

- Check that the database path exists and is writable
- Run with `-v` flag for verbose logging
- Check stderr for error messages

### Method not found errors

- Verify the method name matches exactly (case-sensitive)
- Check that you're using JSON-RPC 2.0 format

### Invalid params errors

- Ensure all required parameters are provided
- Check parameter types match the specification
- Verify JSON is valid (use a JSON validator)

## License

AGPL-3.0 - See [LICENSE](../../LICENSE) for details.
