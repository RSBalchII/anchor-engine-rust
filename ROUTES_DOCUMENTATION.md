# Anchor Engine Routes Documentation

**Date:** 2026-04-02
**Status:** ✅ All Routes Operational

---

## 🎨 UI Routes (Single Page Application)

All UI routes serve the same `index.html` file. The React router handles client-side navigation.

| Route | Method | Handler | Description |
|-------|--------|---------|-------------|
| `/` | GET | `root()` | Redirects to `/search` |
| `/search` | GET | `search_ui()` | Main search interface |
| `/settings` | GET | `search_ui()` | Settings page |
| `/dashboard` | GET | `search_ui()` | Dashboard view |
| `/memory` | GET | `search_ui()` | Memory management |
| `/paths` | GET | `search_ui()` | Watch paths configuration |
| `/quarantine` | GET | `search_ui()` | Quarantine view |
| `/favicon.ico` | GET | `favicon()` | Returns 404 (prevents console noise) |

---

## 🔬 API Endpoints

### Health & Stats

| Route | Method | Handler | Description |
|-------|--------|---------|-------------|
| `/health` | GET | `health_check()` | Health check with version info |
| `/stats` | GET | `get_stats()` | Database statistics |
| `/v1/stats` | GET | `get_stats()` | Alias for UI compatibility |
| `/v1/system/status` | GET | `health_check()` | Alias for UI compatibility |
| `/v1/buckets` | GET | `get_buckets()` | List available buckets |

### Memory/Search

| Route | Method | Handler | Description |
|-------|--------|---------|-------------|
| `/v1/memory/search` | POST | `search_memory()` | Search knowledge base |
| `/v1/memory/ingest` | POST | `ingest_memory()` | Ingest content into memory |

### System Management

| Route | Method | Handler | Description |
|-------|--------|---------|-------------|
| `/v1/system/paths` | GET | `list_watch_paths()` | List watched paths |
| `/v1/system/paths/add` | POST | `add_watch_path()` | Add a watch path |
| `/v1/system/paths/remove` | DELETE | `remove_watch_path()` | Remove a watch path |
| `/v1/system/github/ingest` | POST | `ingest_github()` | Ingest GitHub repository |

### Watchdog

| Route | Method | Handler | Description |
|-------|--------|---------|-------------|
| `/v1/watchdog/status` | GET | `watchdog_status()` | Get watchdog status |
| `/v1/watchdog/start` | POST | `watchdog_start()` | Start watchdog service |
| `/v1/watchdog/stop` | POST | `watchdog_stop()` | Stop watchdog service |
| `/v1/watchdog/ingest` | POST | `watchdog_ingest()` | Trigger manual ingest |

### Settings

| Route | Method | Handler | Description |
|-------|--------|---------|-------------|
| `/v1/settings` | GET | `get_settings()` | Get all settings |
| `/v1/settings` | PUT | `save_settings()` | Save settings to JSON |

### Testing & Snapshots

| Route | Method | Handler | Description |
|-------|--------|---------|-------------|
| `/v1/test/snapshot` | POST | `save_snapshot()` | Save UI test snapshot |

### Chat (OpenAI-Compatible)

| Route | Method | Handler | Description |
|-------|--------|---------|-------------|
| `/v1/chat/completions` | POST | `chat_completions()` | OpenAI-compatible chat API |

---

## 📁 File Locations

### Frontend
```
crates/anchor-engine/static/
└── index.html    (All UI routes serve this file)
```

### Backend
```
crates/anchor-engine/src/
└── api.rs        (All route handlers defined here)
```

### Logs & Snapshots
```
anchor-engine-rust/logs/
├── anchor-engine.log.2026-04-02    (Rolling log file)
├── snapshot-before-search.json     (Search before state)
├── snapshot-after-search.json      (Search after state)
└── snapshot-settings-state.json    (Settings state)
```

### Configuration
```
anchor-engine-rust/
└── user_settings.json    (All settings persist here)
```

---

## 🔄 Navigation Flow

```
User opens browser
    ↓
http://localhost:3160/
    ↓
Redirects to /search
    ↓
index.html loads (React app)
    ↓
React Router handles navigation:
  - /search      → SearchPage component
  - /settings    → SettingsPage component
  - /paths       → PathsPage component
  - /quarantine  → QuarantinePage component
  - /dashboard   → DashboardPage component
  - /memory      → MemoryPage component
```

---

## 🧪 Testing All Routes

### Manual Testing
```bash
# Health check
curl http://localhost:3160/health

# UI routes (should all return HTML)
curl http://localhost:3160/search | findstr "Anchor"
curl http://localhost:3160/settings | findstr "Anchor"
curl http://localhost:3160/paths | findstr "Anchor"
curl http://localhost:3160/quarantine | findstr "Anchor"

# API endpoints
curl http://localhost:3160/v1/stats
curl http://localhost:3160/v1/buckets
curl http://localhost:3160/v1/watchdog/status
curl http://localhost:3160/v1/settings
```

### Automated Testing
1. Open Settings page: `http://localhost:3160/settings`
2. Click "🧪 Run Component Tests"
3. All 12 tests execute including snapshot capture
4. Review results in UI and `logs/` directory

---

## 🛠️ Route Handler Implementation

### Example: UI Route Handler
```rust
/// Search UI route - serves the beautiful frontend interface (v5.0.0 UI).
async fn search_ui() -> (axum::http::HeaderMap, Html<&'static str>) {
    use axum::http::HeaderValue;
    let mut headers = axum::http::HeaderMap::new();
    headers.insert("Cache-Control", HeaderValue::from_static("no-cache, no-store, must-revalidate"));
    headers.insert("Pragma", HeaderValue::from_static("no-cache"));
    headers.insert("Expires", HeaderValue::from_static("0"));
    (headers, Html(include_str!("../static/index.html")))
}
```

### Example: API Route Handler
```rust
/// Get watchdog status.
#[debug_handler]
async fn watchdog_status() -> Json<serde_json::Value> {
    let config = Config::load().unwrap_or_default();
    
    Json(serde_json::json!({
        "is_running": config.watcher.auto_start,
        "files_processed": 0,
        "errors": 0,
        "watched_paths": config.watcher.extra_paths,
        "auto_start": config.watcher.auto_start,
        "stability_threshold_ms": config.watcher.stability_threshold_ms
    }))
}
```

---

## ✅ Verification Checklist

- [x] `/` redirects to `/search`
- [x] `/search` returns HTML
- [x] `/settings` returns HTML
- [x] `/paths` returns HTML
- [x] `/quarantine` returns HTML
- [x] `/dashboard` returns HTML
- [x] `/memory` returns HTML
- [x] `/favicon.ico` returns 404 (intentional)
- [x] All API endpoints respond
- [x] Snapshots save to `logs/` directory
- [x] Settings persist to `user_settings.json`
- [x] Watchdog toggles and persists

---

## 🚨 Common Issues

### 404 on UI Routes
**Problem:** Navigating to `/settings` returns 404
**Solution:** Ensure backend has route handler for all UI paths

### 404 on API Endpoints
**Problem:** `/v1/settings` returns 404
**Solution:** Check route registration in `create_router()`

### Chrome Error Pages
**Problem:** `chrome-error://chromewebdata/` errors
**Solution:** Hard refresh browser (`Ctrl+Shift+R`)

### Duplicate Route Panic
**Problem:** `Overlapping method route` panic
**Solution:** Remove duplicate `.route()` calls

---

**All Routes Operational!** 🎉
