# Troubleshooting Guide

This document provides solutions for common issues encountered when using Anchor Engine Rust.

## Common Issues

### 1. Server Won't Start

#### Symptom: Port Already in Use
```
error: Address already in use
```

**Solution:**
```bash
# Find process using port 3160
netstat -ano | findstr :3160

# Kill the process
taskkill /PID <process_id> /F

# Or use a different port
./target/release/anchor-engine --port 3161
```

#### Symptom: Permission Denied
```
error: Permission denied
```

**Solution:**
- Run with appropriate file system permissions
- Ensure the directory is writable
- On Windows, try running as Administrator if needed

#### Symptom: Database Lock Error
```
error: database is locked
```

**Solution:**
```bash
# Remove the lock file if it exists
rm anchor.db-shm anchor.db-wal

# Or use a different database path
./target/release/anchor-engine --db-path ./new_anchor.db
```

### 2. API Requests Failing

#### Symptom: 401 Unauthorized
```
{"error": "Unauthorized", "message": "API key not provided or invalid"}
```

**Solution:**
1. Check that you're sending the API key in the Authorization header:
   ```
   Authorization: Bearer YOUR_API_KEY
   ```
2. Verify the API key in `user_settings.json` matches the one being sent

#### Symptom: 429 Too Many Requests
```
{"error": "Rate limit exceeded", "message": "Too many requests, please try again later"}
```

**Solution:**
- Wait for the rate limit to reset (typically 1 minute)
- Check that your application is respecting rate limits
- Adjust rate limiting settings in configuration if needed

#### Symptom: 503 Service Unavailable
```
{"error": "Service temporarily unavailable", "message": "Database initializing, please wait..."}
```

**Solution:**
- Wait for the database to initialize (usually <30 seconds)
- Check logs for initialization progress
- This is normal during startup

### 3. MCP Server Issues

#### Symptom: MCP Server Won't Start
```
error: Failed to connect to database
```

**Solution:**
1. Ensure the main anchor-engine server is running
2. Verify the database path is correct:
   ```bash
   ./target/release/anchor-mcp --db-path ./anchor.db
   ```

#### Symptom: MCP Tools Not Responding
```
{"jsonrpc": "2.0", "id": 1, "error": {"code": -32603, "message": "Internal error"}}
```

**Solution:**
1. Check that the main engine service is running
2. Verify the service connection in the MCP server
3. Check logs for specific error details

### 4. Search Issues

#### Symptom: No Search Results
```
{"results": [], "total": 0}
```

**Solution:**
1. Verify content has been ingested:
   ```bash
   # Check database stats
   curl http://localhost:3160/stats
   ```
2. Ensure the query matches ingested content
3. Check that the content wasn't filtered out during ingestion

#### Symptom: Slow Search Performance
- Search taking longer than expected

**Solution:**
1. Check system resources (memory, CPU)
2. Verify database size (large databases may be slower)
3. Optimize queries to be more specific
4. Consider increasing memory allocation

### 5. Ingestion Issues

#### Symptom: Content Not Appearing After Ingestion
```
# Ingestion succeeds but search returns no results
```

**Solution:**
1. Verify the ingestion request was successful (200 status)
2. Check that the content was actually written to mirrored_brain/
3. Wait for any background processing to complete
4. Verify the content wasn't filtered out by security checks

#### Symptom: Large File Ingestion Fails
```
error: Content too large
```

**Solution:**
1. Check the content size against limits in configuration
2. Increase the `max_file_size_bytes` in user_settings.json if needed
3. Consider breaking large files into smaller chunks

### 6. Memory Issues

#### Symptom: High Memory Usage
- Process consuming more memory than expected

**Solution:**
1. Check for memory leaks in custom integrations
2. Adjust cache settings in configuration:
   ```json
   {
     "memory": {
       "max_atoms_in_memory": 5000,
       "gc_cooldown_ms": 60000
     }
   }
   ```
3. Restart the service to clear caches
4. Monitor for any unusual memory growth patterns

#### Symptom: Out of Memory Error
```
error: Out of memory
```

**Solution:**
1. Increase available system memory
2. Reduce the amount of concurrent operations
3. Adjust memory settings in configuration
4. Consider the hardware requirements for your use case

## Diagnostic Commands

### Health Check
```bash
curl http://localhost:3160/health
```

### Database Stats
```bash
curl http://localhost:3160/stats
```

### System Information
```bash
# Check process status
tasklist | findstr anchor-engine

# Check port usage
netstat -an | findstr :3160
```

### Log Files
- Check `logs/` directory for detailed logs
- Look for `anchor_engine.log` or similar files
- Enable verbose logging if needed: `RUST_LOG=debug`

## Performance Troubleshooting

### Slow Response Times
1. **Check system resources:**
   - Memory usage
   - CPU utilization
   - Disk I/O performance

2. **Review configuration:**
   - Database settings
   - Memory limits
   - Cache sizes

3. **Optimize queries:**
   - Use more specific search terms
   - Adjust token budgets
   - Consider search mode (standard vs max-recall)

### High Resource Usage
1. **Monitor continuously:**
   - Use system monitoring tools
   - Check for memory leaks
   - Monitor for resource growth over time

2. **Adjust settings:**
   - Reduce cache sizes
   - Limit concurrent operations
   - Adjust garbage collection settings

## Recovery Procedures

### Database Corruption
1. Stop the service
2. Make a backup of the database file
3. Attempt repair with SQLite tools:
   ```bash
   sqlite3 anchor.db "PRAGMA integrity_check;"
   ```
4. If repair fails, restore from backup or re-ingest content

### Service Crash
1. Check logs for error details
2. Verify system resources are available
3. Restart the service
4. Monitor for recurring crashes

### Configuration Issues
1. Verify `user_settings.json` syntax (valid JSON)
2. Compare with `user_settings.json.template`
3. Restart service after configuration changes
4. Check logs for configuration-related errors

## When to Seek Help

Contact support or community if you encounter:
- Persistent crashes that don't match known issues
- Performance problems that can't be resolved with optimization
- Security-related concerns
- Feature requests or enhancement suggestions

## Related Documentation

- [API Reference](../api/reference.md) - API error codes and responses
- [Setup Guide](../setup/installation.md) - Installation troubleshooting
- [Performance Guide](../technical/performance.md) - Performance optimization
- [Architecture Spec](../../specs/current-standards/001-architecture-spec.md) - System architecture details