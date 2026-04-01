# API Reference

This document provides a comprehensive reference for the Anchor Engine HTTP API endpoints.

## Base URL

All API endpoints are served from: `http://localhost:3160` (default port, configurable)

## Authentication

Most endpoints require an API key in the `Authorization` header:

```
Authorization: Bearer YOUR_API_KEY
```

## Endpoints

### Health Check

**GET** `/health`

Check the health status of the Anchor Engine.

#### Response
```json
{
  "status": "healthy",
  "timestamp": "2026-03-31T03:30:35.331Z",
  "message": "Anchor Context Engine is running and database is responsive"
}
```

### Search

**POST** `/v1/memory/search`

Perform semantic search across the knowledge base.

#### Request Body
```json
{
  "query": "your search query",
  "max_chars": 5000,
  "max_results": 50,
  "mode": "combined",
  "budget": {
    "planet_budget": 0.7,
    "moon_budget": 0.3,
    "total_tokens": 8192,
    "max_recall": false
  }
}
```

#### Response
```json
{
  "metadata": {
    "totalResults": 5,
    "durationMs": 123,
    "strategy": "combined"
  },
  "results": [
    {
      "uuid": "atom-id",
      "content": "retrieved content",
      "source": "source path",
      "timestamp": "2026-03-30T14:23:36.487Z",
      "score": 0.85,
      "tags": ["#tag1", "#tag2"],
      "buckets": ["inbox"],
      "provenance": "internal",
      "compound_id": "compound-id",
      "start_byte": 0,
      "end_byte": 100
    }
  ]
}
```

### Ingest Content

**POST** `/v1/memory/ingest`

Ingest new content into the knowledge base.

#### Request Body
```json
{
  "source": "document.md",
  "content": "content to ingest",
  "bucket": "inbox",
  "options": {
    "sanitize": true,
    "extract_tags": true,
    "max_keywords": 10
  }
}
```

#### Response
```json
{
  "source_id": "generated-source-id",
  "atoms_created": 5,
  "atom_ids": [1, 2, 3, 4, 5],
  "tags": ["#tag1", "#tag2"]
}
```

### Distill Knowledge

**POST** `/v1/memory/distill`

Run radial distillation to compress knowledge into decision records.

#### Request Body
```json
{
  "seed": "optional seed query",
  "radius": 2,
  "output_format": "json"
}
```

#### Response
```json
{
  "output_path": "./distills/distilled_2026-03-31.json",
  "compression_ratio": 1.5,
  "total_atoms": 100,
  "total_sources": 5,
  "duration_ms": 2500
}
```

### Illuminate (BFS Traversal)

**POST** `/v1/memory/illuminate`

Perform BFS graph traversal to discover related concepts.

#### Request Body
```json
{
  "seed": "starting query",
  "depth": 2,
  "max_nodes": 50
}
```

#### Response
```json
{
  "nodes": [
    {
      "id": 123,
      "source_path": "/path/to/source",
      "content": "node content",
      "tags": ["#tag1"],
      "hop_distance": 1,
      "gravity_score": 0.85,
      "simhash": "1234567890abcdef"
    }
  ],
  "total": 5,
  "nodes_explored": 10,
  "duration_ms": 150
}
```

### System Statistics

**GET** `/stats`

Get database statistics and system information.

#### Response
```json
{
  "atoms": 151876,
  "sources": 436,
  "tags": 280000,
  "molecules": 0
}
```

### Add Watch Path

**POST** `/v1/system/paths/add`

Add a directory to be monitored for new content.

#### Request Body
```json
{
  "path": "/path/to/watch"
}
```

#### Response
```json
{
  "success": true,
  "message": "Path added to watch list"
}
```

### Remove Watch Path

**DELETE** `/v1/system/paths/remove`

Remove a directory from the watch list.

#### Request Body
```json
{
  "path": "/path/to/remove"
}
```

#### Response
```json
{
  "success": true,
  "message": "Path removed from watch list"
}
```

## Error Responses

All error responses follow this format:

```json
{
  "error": "Error message",
  "details": "Additional details"
}
```

Common HTTP status codes:
- `200`: Success
- `400`: Bad request
- `401`: Unauthorized
- `404`: Not found
- `500`: Internal server error
- `503`: Service temporarily unavailable

## Rate Limiting

The API implements rate limiting:
- General requests: 100 requests/minute per IP
- Ingest operations: 20 requests/minute per IP

Rate limit headers are included in responses:
- `X-RateLimit-Limit`: Request limit
- `X-RateLimit-Remaining`: Remaining requests
- `X-RateLimit-Reset`: Time when counter resets

## Streaming Responses

Some endpoints support streaming responses via Server-Sent Events (SSE):

- `/v1/memory/search` with `stream=true` query parameter
- `/v1/memory/distill` with `stream=true` query parameter

For streaming endpoints, the response format is different and sent as SSE events.

## Related Documentation

- [Architecture Spec](../../specs/spec.md) - System architecture details
- [Performance Guide](../technical/performance.md) - Performance optimization
- [Troubleshooting](../troubleshooting/common-issues.md) - Common issues and solutions