# Carnelia Sync Server

WebSocket relay server for real-time document collaboration using Carnelia CRDTs.

## Overview

This server does **NOT** process CRDT logic - it simply relays serialized state between connected clients. All merge operations happen client-side via the WASM CRDT library. This keeps the server lightweight and stateless.

## Architecture

```
┌─────────────┐     WebSocket      ┌─────────────┐
│  Browser 1  │◄──────────────────►│             │
│   (WASM)    │                    │    Sync     │
└─────────────┘     WebSocket      │   Server    │
                                   │             │
┌─────────────┐     WebSocket      │  (Relay)    │
│  Browser 2  │◄──────────────────►│             │
│   (WASM)    │                    └─────────────┘
└─────────────┘
```

- **Rooms**: Clients join "rooms" based on document ID
- **State Relay**: When a client sends CRDT state, it's broadcast to others in the room
- **Presence**: Cursor and selection positions are relayed between clients

## Quick Start

```bash
# Install dependencies
npm install

# Development (with hot reload)
npm run dev

# Production build
npm run build
npm start
```

## API Documentation

Once running, visit **http://localhost:3001/docs** for the interactive Swagger UI.

## Endpoints

### HTTP

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/stats` | GET | Server statistics |
| `/rooms` | GET | List active rooms |
| `/rooms/:docId` | GET | Get room details |
| `/docs` | GET | Swagger UI |

### WebSocket

Connect to `ws://localhost:3001/ws`

#### Message Types

**join** - Join a document room
```json
{
  "type": "join",
  "docId": "doc-123",
  "userId": "user-abc",
  "userName": "Alice",
  "userColor": "#FF6B6B"
}
```

**sync** - Send document state
```json
{
  "type": "sync",
  "docId": "doc-123",
  "userId": "user-abc",
  "state": "<serialized CRDT>"
}
```

**presence** - Update cursor/selection
```json
{
  "type": "presence",
  "docId": "doc-123",
  "userId": "user-abc",
  "userName": "Alice",
  "userColor": "#FF6B6B",
  "cursor": 42,
  "selectionStart": null,
  "selectionEnd": null
}
```

**sync_request** - Request state from peers
```json
{
  "type": "sync_request",
  "docId": "doc-123",
  "userId": "user-abc"
}
```

#### Server Messages

**room_users** - Current users in room (sent after join)
```json
{
  "type": "room_users",
  "users": [
    { "userId": "...", "userName": "...", "userColor": "..." }
  ]
}
```

**user_joined** - New user joined
```json
{
  "type": "user_joined",
  "userId": "...",
  "userName": "...",
  "userColor": "..."
}
```

**user_left** - User left
```json
{
  "type": "user_left",
  "userId": "..."
}
```

## Configuration

Environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `3001` | Server port |
| `CORS_ORIGIN` | `*` | CORS allowed origin |

## Deployment

### Railway

1. Push code to GitHub
2. Create new project in Railway
3. Connect to repository
4. Set environment variables:
   - `PORT` (Railway provides this automatically)
   - `CORS_ORIGIN` (your Vercel frontend URL)

### Docker

```dockerfile
FROM node:20-slim
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY dist ./dist
EXPOSE 3001
CMD ["node", "dist/index.js"]
```

## Development

```bash
# Type checking
npm run lint

# Watch mode
npm run dev
```

## License

MIT
