# MCP Server Standard

**Standard ID:** MCP-001  
**Status:** ✅ Implemented  
**Created:** March 30, 2026  
**Updated:** March 30, 2026  

---

## Pain Point

**Problem:** AI agents cannot interact with the Rust Anchor Engine programmatically.

**Context:** The Node.js version has a fully-functional MCP server that allows Claude and other AI agents to:
- Query the knowledge graph
- Ingest new content
- Retrieve files
- Get database statistics

The Rust version, despite having a complete core engine, lacks this integration layer. This blocks:
- Agent-assisted development
- Automated knowledge management
- Integration with OpenCLAW and other agent frameworks

**Impact:** High - Without MCP, the Rust engine is isolated and cannot participate in the agent ecosystem.

---

## Requirements

### Functional Requirements

1. **JSON-RPC 2.0 Protocol**
   - Must implement standard JSON-RPC 2.0 request/response format
   - Must support batch requests (optional)
   - Must return proper error codes (-32700 to -32000)

2. **Stdio Communication**
   - Must read requests from stdin
   - Must write responses to stdout
   - Must handle EOF gracefully (client disconnect)
   - Must flush stdout after each response

3. **Tool Exposure**
   - Must expose all core engine functions as MCP tools:
     - `anchor_query` - STAR algorithm search
     - `anchor_distill` - Knowledge distillation
     - `anchor_illuminate` - BFS graph traversal
     - `anchor_read_file` - File content retrieval
     - `anchor_list_compounds` - List molecules
     - `anchor_get_stats` - Database statistics
     - `anchor_ingest_text` - Ingest raw text
     - `anchor_ingest_file` - Ingest from file

4. **Error Handling**
   - Must parse JSON-RPC requests safely
   - Must return structured error objects
   - Must log errors to stderr (not stdout)
   - Must continue processing after errors (no crashes)

5. **Async Support**
   - Must use tokio async runtime
   - Must handle concurrent requests (via Arc<Mutex>)
   - Must not block the async runtime on I/O

### Non-Functional Requirements

1. **Performance**
   - Request parsing: <1ms
   - Response serialization: <1ms
   - Tool execution: depends on tool (query ~50-200ms)

2. **Memory**
   - Base memory usage: <50MB
   - Per-request overhead: <1MB
   - No memory leaks on repeated calls

3. **Compatibility**
   - Must be compatible with Node.js MCP server protocol
   - Must work with existing agent harnesses (OpenCLAW, etc.)
   - Must support stdio transport (no HTTP required)

---

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      AI Agent (Claude)                       │
└────────────────────┬────────────────────────────────────────┘
                     │ JSON-RPC 2.0 (stdio)
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                    anchor-mcp (this crate)                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  main.rs                                               │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │  JsonRpcRequest  │  JsonRpcResponse             │  │  │
│  │  │  - jsonrpc       │  - jsonrpc                   │  │  │
│  │  │  - id            │  - id                        │  │  │
│  │  │  - method        │  - result                    │  │  │
│  │  │  - params        │  - error                     │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  │                                                        │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │  McpServer                                       │  │  │
│  │  │  - service: Arc<Mutex<AnchorService>>           │  │  │
│  │  │  - handle_request()                             │  │  │
│  │  │  - handle_query()                               │  │  │
│  │  │  - handle_ingest_text()                         │  │  │
│  │  │  - handle_ingest_file()                         │  │  │
│  │  │  - ... (all tool handlers)                      │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────────────┘  │
└────────────────────┬────────────────────────────────────────┘
                     │ Method calls
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              anchor-engine (core crate)                      │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  AnchorService                                         │  │
│  │  - search(query, max_results)                         │  │
│  │  - ingest(source, content, bucket)                    │  │
│  │  - read_file(path, start_line, end_line)              │  │
│  │  - get_stats()                                        │  │
│  │  - ...                                                 │  │
│  └───────────────────────────────────────────────────────┘  │
│                          │                                   │
│                          ▼                                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Database (SQLite)                                     │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Request Path:**
   ```
   Agent → stdin → JsonRpcRequest → McpServer::handle_request() 
   → AnchorService method → Result<Value>
   ```

2. **Response Path:**
   ```
   Result<Value> → JsonRpcResponse → serialize → stdout → Agent
   ```

3. **Error Path:**
   ```
   Error → JsonRpcError → JsonRpcResponse { error: Some(...) } 
   → serialize → stdout → Agent
   ```

---

## Implementation

### File Structure

```
crates/anchor-mcp/
├── Cargo.toml              # Package configuration
├── README.md               # User documentation
├── src/
│   └── main.rs             # MCP server implementation
└── tests/
    └── mcp_integration_test.rs  # Integration tests
```

### Dependencies

```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
clap = { version = "4.4", features = ["derive"] }
anchor-engine = { path = "../anchor-engine" }
```

### Key Code Patterns

#### Request Handler

```rust
async fn handle_request(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
    match request.method.as_str() {
        "anchor_query" => self.handle_query(&request.id, &request.params).await,
        "anchor_get_stats" => self.handle_get_stats(&request.id).await,
        // ... other methods
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: None,
            error: Some(JsonRpcError {
                code: METHOD_NOT_FOUND,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    }
}
```

#### Tool Implementation

```rust
async fn handle_query(&self, id: &Value, params: &Value) -> JsonRpcResponse {
    let params: QueryParams = match serde_json::from_value(params.clone()) {
        Ok(p) => p,
        Err(e) => return self.error_response(id, INVALID_PARAMS, &format!("Invalid params: {}", e)),
    };

    let service = self.service.lock().await;
    
    match service.search(params.query, params.max_results).await {
        Ok(response) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: id.clone(),
            result: Some(serde_json::to_value(response).unwrap()),
            error: None,
        },
        Err(e) => self.error_response(id, APPLICATION_ERROR, &e.to_string()),
    }
}
```

#### Stdio Loop

```rust
let stdin = io::stdin();
let mut stdout = io::stdout();
let mut line = String::new();

loop {
    line.clear();
    match stdin.lock().read_line(&mut line) {
        Ok(0) => break, // EOF
        Ok(_) => {
            let request = serde_json::from_str(&line.trim())?;
            let response = server.handle_request(&request).await;
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }
        Err(e) => { /* handle error */ }
    }
}
```

---

## Testing

### Test Categories

1. **Unit Tests** (not applicable - main.rs only)

2. **Integration Tests**
   - Server starts successfully
   - Valid requests return valid responses
   - Invalid requests return error responses
   - Tool calls execute correctly
   - EOF handling works

3. **End-to-End Tests** (future)
   - Test with real AI agent (Claude)
   - Test with OpenCLAW harness
   - Performance benchmarks

### Example Test

```rust
#[test]
fn test_anchor_get_stats() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut child = spawn_mcp_server(&db_path);
    std::thread::sleep(std::time::Duration::from_millis(500));

    let response: Value = send_request(&mut child, "anchor_get_stats", json!({}));

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert!(response["result"]["atoms"].is_number());

    child.kill().expect("Failed to kill server");
}
```

### Running Tests

```bash
# Run all tests
cargo test --package anchor-mcp

# Run with output
cargo test --package anchor-mcp -- --nocapture

# Run specific test
cargo test --package anchor-mcp test_anchor_get_stats
```

---

## Usage

### Building

```bash
cd /data/data/com.termux/files/home/projects/anchor-engine-rust
cargo build --release --package anchor-mcp
```

### Running

```bash
# Default database path
./target/release/anchor-mcp

# Custom database path
./target/release/anchor-mcp --db-path /path/to/anchor.db

# Verbose logging
./target/release/anchor-mcp --db-path ./anchor.db -v
```

### Example Session

```bash
# Start server
./target/release/anchor-mcp --db-path ./anchor.db

# Send request (in another terminal or via pipe)
echo '{"jsonrpc":"2.0","id":1,"method":"anchor_get_stats","params":{}}' | ./target/release/anchor-mcp

# Expected response
{"jsonrpc":"2.0","id":1,"result":{"atoms":0,"molecules":0,"sources":0,"tags":0}}
```

---

## Troubleshooting

### Common Issues

1. **Server won't start**
   - Check database path exists and is writable
   - Run with `-v` flag for verbose logging
   - Check stderr for error messages

2. **Parse errors**
   - Verify JSON is valid (use jsonlint.com)
   - Ensure request has `jsonrpc`, `id`, `method` fields
   - Check for trailing commas or unquoted keys

3. **Method not found**
   - Verify method name matches exactly (case-sensitive)
   - Check method is implemented in McpServer

4. **Invalid params**
   - Ensure all required parameters provided
   - Check parameter types match specification
   - Verify JSON structure matches expected schema

### Debugging

```bash
# Enable verbose logging
RUST_LOG=debug ./target/release/anchor-mcp -v

# Capture stderr to file
./target/release/anchor-mcp 2> mcp.log

# Use strace for system call tracing (Linux)
strace -f ./target/release/anchor-mcp
```

---

## Future Enhancements

### Short-Term
- [ ] Implement full `anchor_distill` functionality
- [ ] Implement full `anchor_illuminate` functionality
- [ ] Add batch request support
- [ ] Add request timeout handling

### Medium-Term
- [ ] Add HTTP transport option (for non-stdio environments)
- [ ] Add WebSocket support (for browser agents)
- [ ] Add authentication/authorization
- [ ] Add request rate limiting

### Long-Term
- [ ] Add streaming responses (for large result sets)
- [ ] Add progress notifications (for long operations)
- [ ] Add tool discovery endpoint
- [ ] Add MCP specification compliance testing

---

## References

- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Model Context Protocol (MCP)](https://modelcontextprotocol.io/)
- [anchor-engine-node MCP Server](../../AEN/engine/src/mcp/)
- [tokio Documentation](https://docs.rs/tokio)
- [serde_json Documentation](https://docs.rs/serde_json)

---

## Changelog

### v0.1.0 (March 30, 2026)
- Initial implementation
- JSON-RPC 2.0 protocol handler
- 8 MCP tools (6 full, 2 stubs)
- Integration test suite
- Comprehensive documentation
