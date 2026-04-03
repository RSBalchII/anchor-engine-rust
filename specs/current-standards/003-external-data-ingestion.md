# Standard 003: External Data Ingestion

**Date**: April 1, 2026  
**Status**: Active  
**Supersedes**: N/A

---

## Principle

**All external data MUST pass through a Data Transfer Object (DTO) layer before reaching internal business logic.**

---

## Rationale

External APIs (GitHub, webhooks, third-party services) have:
- Deeply nested, complex schemas
- Ever-changing field structures
- Inconsistent nullability
- Large payloads with mostly irrelevant data

Directly deserializing external JSON into internal domain models creates:
- **Fragile code** - One API change breaks your handlers
- **Type errors** - Axum's `Handler` trait requires exact `DeserializeOwned` bounds
- **Security risks** - No validation at the boundary
- **Maintenance burden** - Update dozens of structs for every schema change

---

## Standard

### 1. Define DTOs for External Data

```rust
/// Simplified, validated representation of external payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubIngestDto {
    pub repo_url: String,
    pub branch: String,
    pub owner: String,
    pub repo_name: String,
    pub sender: String,
}
```

**Rules**:
- ✅ Flat structure (no nested objects)
- ✅ Only fields you actually need
- ✅ All fields use owned types (`String`, not `&str`)
- ✅ Derive `Serialize` and `Deserialize`

### 2. Implement Custom `FromRequest` Extractor

```rust
use axum::{
    async_trait,
    body::Bytes,
    extract::{FromRequest, Request},
    http::StatusCode,
};
use serde_json::Value;

#[async_trait]
impl<S> FromRequest<S> for GithubIngestDto
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, _state: &S) -> Result<Self, Self::Rejection> {
        // 1. Read raw body bytes
        let bytes = Bytes::from_request(req, _state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Body read error: {}", e)))?;

        // 2. Parse to generic JSON Value (flexible, won't fail on unexpected fields)
        let json: Value = serde_json::from_slice(&bytes)
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid JSON: {}", e)))?;

        // 3. Extract ONLY required fields with safe defaults
        let repo_url = json["repository"]["html_url"]
            .as_str()
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing repository.html_url".into()))?
            .to_string();
        
        let owner = json["repository"]["owner"]["login"]
            .as_str()
            .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing repository.owner.login".into()))?
            .to_string();
        
        let branch = json["repository"]["default_branch"]
            .as_str()
            .unwrap_or("main")  // Safe default
            .to_string();

        // 4. Validate
        if repo_url.is_empty() {
            return Err((StatusCode::BAD_REQUEST, "Repository URL cannot be empty".into()));
        }

        // 5. Return validated DTO
        Ok(GithubIngestDto {
            repo_url,
            branch,
            owner,
            // ...
        })
    }
}
```

**Rules**:
- ✅ Parse to `serde_json::Value` first (flexible)
- ✅ Extract fields manually with `.as_str()`, `.as_i64()`, etc.
- ✅ Provide safe defaults with `.unwrap_or()`
- ✅ Validate required fields explicitly
- ✅ Return clear error messages

### 3. Use DTO in Handler

```rust
#[axum::debug_handler]
async fn ingest_github(
    State(state): State<SharedState>,
    dto: GithubIngestDto,  // ← Must be LAST argument (body extractor)
) -> Result<Json<Response>, (StatusCode, String)> {
    // dto is GUARANTEED valid here
    // No deserialization errors possible
    
    service.github_service.sync_repo(&dto.repo_url).await?;
    
    Ok(Json(Response { success: true }))
}
```

**Rules**:
- ✅ DTO must be **last argument** (consumes request body)
- ✅ Handler receives validated data
- ✅ No additional validation needed in handler

---

## Benefits

### 1. Isolation from Schema Changes

When GitHub changes their API:
- ❌ **Without DTO**: Update 20 structs, recompile everything
- ✅ **With DTO**: Update only the extractor, internal logic unchanged

### 2. Clear Error Messages

```
❌ Without DTO:
"the trait bound `Handler<_, _>` is not satisfied"

✅ With DTO:
"Missing repository.owner.login"
```

### 3. Validation at Boundary

External data is untrusted. Validate once, at the boundary, before it touches your business logic.

### 4. Simplified Handlers

Handlers focus on business logic, not JSON parsing.

---

## Anti-Patterns

### ❌ Direct Deserialization

```rust
// DON'T DO THIS
#[derive(Deserialize)]
struct GitHubWebhook {
    repository: Repository,
    sender: Sender,
    // ... 50 more nested fields
}

#[debug_handler]
async fn handler(Json(webhook): Json<GitHubWebhook>) {
    // Fragile! Any schema change breaks this
}
```

### ❌ No Validation

```rust
// DON'T DO THIS
let repo_url = json["repository"]["html_url"].as_str().unwrap();
// Panics if field is missing!
```

### ❌ DTO in Middle of Arguments

```rust
// DON'T DO THIS - won't compile!
async fn handler(
    dto: GithubIngestDto,  // ← Body extractor must be LAST
    State(state): State<SharedState>,
) { }
```

---

## When to Use

| Scenario | Use DTO? |
|----------|----------|
| GitHub webhooks | ✅ Yes |
| Third-party API responses | ✅ Yes |
| User-uploaded JSON | ✅ Yes |
| Internal service-to-service | ⚠️ Optional |
| Simple key-value config | ❌ No |

**Rule of thumb**: If the schema is complex, nested, or outside your control → **DTO required**.

---

## Related Standards

- **Standard 001**: Architecture Spec - Service boundaries
- **Standard 004**: Secure Credential Storage
- **Code Style**: Error handling patterns

---

## Enforcement

- Code review checklist: "Does this external data handler use a DTO?"
- CI lint rule: Warn on direct `Json<T>` deserialization for complex types
- Architecture decision record: Document all external data sources

---

## Appendix: Complete Example

See implementation in:
- `crates/anchor-engine/src/dto.rs` - DTO definitions
- `crates/anchor-engine/src/extractors/github.rs` - Extractor implementation
- `crates/anchor-engine/src/api.rs` - Handler usage
