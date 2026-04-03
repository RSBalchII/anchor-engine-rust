# Standard 004: Secure Credential Storage

**Date**: April 1, 2026  
**Status**: Active  
**Supersedes**: N/A

---

## Principle

**Secrets (API tokens, passwords, keys) MUST be stored in OS-managed encrypted credential storage, never in plaintext files, environment variables, or logs.**

---

## Rationale

Credentials are the keys to your kingdom. Leaks happen through:
- **Accidental commits** - `.env` files in git
- **Log files** - Tokens printed to console or log files
- **Process inspection** - Environment variables visible via `/proc/[pid]/environ`
- **Core dumps** - Memory dumps contain heap strings
- **AI agents** - Models can accidentally echo tokens in responses

OS-managed credential storage provides:
- **Encryption at rest** - Windows Credential Manager, macOS Keychain, Linux libsecret
- **Access control** - Only your application can read credentials
- **No plaintext** - Never written to disk unencrypted
- **Audit trail** - Some systems log credential access

---

## Standard

### 1. Use `keyring` Crate

```toml
[dependencies]
keyring = "2.3"
```

**Why keyring?**
- Cross-platform (Windows, macOS, Linux)
- Uses native OS credential storage
- Simple API
- Actively maintained

### 2. Service Structure

```rust
use keyring::Entry;

pub struct GitHubService {
    credential_entry: Entry,
    // ... other fields
}

impl GitHubService {
    pub fn new() -> Result<Self> {
        // Create credential entry (service="AnchorEngine", user="GitHub")
        let credential_entry = Entry::new("AnchorEngine", "GitHub")
            .map_err(|e| CredentialError::CreateFailed(e.to_string()))?;
        
        Ok(Self {
            credential_entry,
            // ...
        })
    }
}
```

**Rules**:
- ✅ Use service name + username as key
- ✅ Initialize in constructor
- ✅ Handle creation errors gracefully

### 3. Store Credentials

```rust
pub fn store_credentials(&self, token: &str) -> Result<()> {
    self.credential_entry.set_password(token)
        .map_err(|e| CredentialError::StoreFailed(e.to_string()))?;
    
    info!("✅ Credentials stored securely");
    Ok(())
}
```

**Rules**:
- ✅ Never log the token value
- ✅ Log only success/failure
- ✅ Return clear errors

### 4. Retrieve Credentials

```rust
pub fn get_credentials(&self) -> Option<String> {
    // Try secure storage first
    if let Ok(token) = self.credential_entry.get_password() {
        return Some(token);
    }
    
    // Fallback to environment variable (for CLI/CI usage)
    std::env::var("GITHUB_TOKEN").ok()
}
```

**Rules**:
- ✅ Secure storage is primary source
- ✅ Environment variable is fallback (not primary)
- ✅ Return `Option<String>`, not `Result` (missing credentials is OK)

### 5. Delete Credentials

```rust
pub fn delete_credentials(&self) -> Result<()> {
    self.credential_entry.delete_password()
        .map_err(|e| CredentialError::DeleteFailed(e.to_string()))?;
    
    info!("🗑️  Credentials deleted");
    Ok(())
}
```

### 6. Never Log Credentials

```rust
// ❌ NEVER DO THIS
info!("Using token: {}", token);  // LEAKS TOKEN!

// ✅ DO THIS
info!("Using stored credentials");  // No token value

// ✅ OR THIS
debug!("Authenticating with {} chars", token.len());  // Only length
```

### 7. Never Serialize Credentials

```rust
// ❌ NEVER DO THIS
#[derive(Serialize)]
struct ApiResponse {
    token: String,  // LEAKS TOKEN!
}

// ✅ DO THIS
#[derive(Serialize)]
struct ApiResponse {
    has_credentials: bool,
    message: String,
    // token field omitted
}
```

---

## Fallback Strategy

| Priority | Source | Use Case |
|----------|--------|----------|
| 1 | OS Credential Manager | Primary (desktop users) |
| 2 | Environment variable | Fallback (CI/CLI users) |
| 3 | None | Public repos only |

**Implementation**:
```rust
pub fn get_token(&self) -> Option<String> {
    self.get_credentials()  // Secure storage
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())  // Env var
}
```

---

## MCP Integration

For AI agent integration, expose credential management via MCP tools:

```rust
// MCP tool: anchor_github_credentials
async fn handle_github_credentials(&self, params: Params) -> Result<Value> {
    match params.action.as_str() {
        "check" => {
            let has_creds = self.service.has_credentials();
            Ok(json!({
                "has_credentials": has_creds,
                "message": if has_creds { 
                    "GitHub credentials configured" 
                } else { 
                    "No GitHub credentials found" 
                }
            }))
        }
        "set" => {
            self.service.store_credentials(&params.token)?;
            Ok(json!({"success": true, "message": "Credentials stored securely"}))
        }
        "delete" => {
            self.service.delete_credentials()?;
            Ok(json!({"success": true, "message": "Credentials deleted"}))
        }
        _ => Err(anyhow!("Unknown action")),
    }
}
```

**Rules**:
- ✅ Token never appears in response
- ✅ Only success/failure messages
- ✅ AI agents can manage credentials safely

---

## Anti-Patterns

### ❌ Plaintext Config Files

```toml
# config.toml - DON'T DO THIS
github_token = "ghp_abc123..."  # VISIBLE IN PLAINTEXT!
```

### ❌ Hardcoded Tokens

```rust
// DON'T DO THIS
const GITHUB_TOKEN: &str = "ghp_abc123...";  // COMMITTED TO GIT!
```

### ❌ Logging Tokens

```rust
// DON'T DO THIS
debug!("GitHub API request with token: {}", token);  // LEAKS TO LOGS!
```

### ❌ Environment Variable as Primary

```rust
// DON'T DO THIS - env var should be fallback only
let token = std::env::var("GITHUB_TOKEN")
    .expect("GITHUB_TOKEN must be set");  // No secure storage!
```

### ❌ Returning Tokens in API Responses

```rust
// DON'T DO THIS
#[derive(Serialize)]
struct Status {
    token: String,  // LEAKS IN API RESPONSE!
}
```

---

## Security Checklist

- [ ] Credentials stored in OS credential manager
- [ ] Token never logged (not even debug logs)
- [ ] Token never serialized (not even in internal structs)
- [ ] Token never returned in API responses
- [ ] Environment variable is fallback only
- [ ] MCP tools abstract token away from AI agents
- [ ] Error messages don't leak token info

---

## Related Standards

- **Standard 003**: External Data Ingestion - DTO pattern
- **Code Style**: Logging standards
- **Testing**: Mock credential storage for tests

---

## Implementation Examples

See:
- `crates/anchor-engine/src/services/github.rs` - Full implementation
- `crates/anchor-mcp/src/main.rs` - MCP tool integration
- `crates/anchor-engine/Cargo.toml` - Dependencies

---

## Compliance

All new code that handles credentials MUST:
1. Use `keyring` crate for storage
2. Never log token values
3. Never serialize tokens
4. Provide environment variable fallback
5. Expose via MCP tools (if applicable)

Code review checklist: "Are credentials stored securely per Standard 004?"
