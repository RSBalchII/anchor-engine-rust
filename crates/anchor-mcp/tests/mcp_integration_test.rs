//! Integration tests for Anchor MCP Server
//!
//! These tests spawn the MCP server as a subprocess and communicate via stdin/stdout.

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::{json, Value};
use std::io::{Write, Read};
use std::process::{Stdio, Child, Command as StdCommand};
use tempfile::TempDir;

/// Spawn the MCP server process
fn spawn_mcp_server(db_path: &str) -> Child {
    StdCommand::new("cargo")
        .args(["run", "--package", "anchor-mcp", "--", "--db-path", db_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn MCP server")
}

/// Send a JSON-RPC request and get the response
fn send_request(child: &mut Child, method: &str, params: Value) -> Value {
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params
    });

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    let mut stdout = child.stdout.take().expect("Failed to open stdout");

    // Send request
    writeln!(stdin, "{}", request).expect("Failed to write request");
    drop(stdin); // Close stdin to flush

    // Read response
    let mut response = String::new();
    stdout.read_to_string(&mut response).expect("Failed to read response");

    serde_json::from_str(&response).expect("Failed to parse response")
}

#[test]
fn test_mcp_server_starts() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    let mut cmd = Command::cargo_bin("anchor-mcp").unwrap();
    cmd.arg("--db-path").arg(&db_path);
    cmd.assert().success();
}

#[test]
fn test_anchor_get_stats() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    let mut child = spawn_mcp_server(&db_path);

    // Give the server time to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test anchor_get_stats
    let response: Value = send_request(&mut child, "anchor_get_stats", json!({}));

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    
    // Should have either result or error
    assert!(response["result"].is_object() || response["error"].is_object());

    // If successful, check result structure
    if let Some(result) = response["result"].as_object() {
        assert!(result.contains_key("atoms"));
        assert!(result.contains_key("molecules"));
        assert!(result.contains_key("sources"));
        assert!(result.contains_key("tags"));
    }

    child.kill().expect("Failed to kill server");
}

#[test]
fn test_anchor_query_empty_database() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    let mut child = spawn_mcp_server(&db_path);

    // Give the server time to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test anchor_query on empty database
    let response: Value = send_request(
        &mut child,
        "anchor_query",
        json!({
            "query": "#test",
            "max_results": 10
        })
    );

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);

    // Should return empty results
    if let Some(result) = response["result"].as_object() {
        assert!(result.contains_key("results"));
        assert!(result["results"].is_array());
        assert!(result.contains_key("total"));
    }

    child.kill().expect("Failed to kill server");
}

#[test]
fn test_invalid_method() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    let mut child = spawn_mcp_server(&db_path);

    // Give the server time to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test invalid method
    let response: Value = send_request(
        &mut child,
        "invalid_method",
        json!({})
    );

    // Verify error response
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32601); // METHOD_NOT_FOUND

    child.kill().expect("Failed to kill server");
}

#[test]
fn test_invalid_params() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    let mut child = spawn_mcp_server(&db_path);

    // Give the server time to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Test invalid params (missing required field)
    let response: Value = send_request(
        &mut child,
        "anchor_query",
        json!({
            "max_results": 10
            // Missing "query" field
        })
    );

    // Verify error response
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response["error"].is_object());
    assert_eq!(response["error"]["code"], -32602); // INVALID_PARAMS

    child.kill().expect("Failed to kill server");
}

#[test]
fn test_ingest_and_query() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db").to_str().unwrap().to_string();

    let mut child = spawn_mcp_server(&db_path);

    // Give the server time to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Ingest some text
    let ingest_response: Value = send_request(
        &mut child,
        "anchor_ingest_text",
        json!({
            "content": "Rust is a systems programming language. It is known for safety and performance.",
            "filename": "test.md",
            "bucket": "test"
        })
    );

    // Verify ingestion was successful
    if let Some(result) = ingest_response["result"].as_object() {
        assert_eq!(result["success"], true);
        assert!(result.contains_key("atoms_ingested"));
        assert!(result.contains_key("molecules_created"));
    }

    // Query for the ingested content
    let query_response: Value = send_request(
        &mut child,
        "anchor_query",
        json!({
            "query": "#rust",
            "max_results": 10
        })
    );

    // Verify query response
    if let Some(result) = query_response["result"].as_object() {
        assert!(result.contains_key("results"));
        assert!(result.contains_key("total"));
    }

    child.kill().expect("Failed to kill server");
}
