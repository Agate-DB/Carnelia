/**
 * Carnelia Sync Server
 * 
 * WebSocket relay server for real-time document collaboration.
 * This server does NOT process CRDT logic - it simply relays
 * serialized state between connected clients. All merge operations
 * happen client-side via the WASM CRDT library.
 * 
 * Architecture:
 * - Clients connect to a "room" (document ID)
 * - When a client sends state, it's broadcast to all other clients in the room
 * - Presence (cursor/selection) updates are also relayed
 */

import express from 'express';
import cors from 'cors';
import { createServer } from 'http';
import { WebSocketServer, WebSocket } from 'ws';
import swaggerUi from 'swagger-ui-express';
import swaggerJsdoc from 'swagger-jsdoc';

// =============================================================================
// Configuration
// =============================================================================

const PORT = process.env.PORT || 3001;
const CORS_ORIGIN = process.env.CORS_ORIGIN || '*';

// =============================================================================
// Types
// =============================================================================

/**
 * Message types for WebSocket communication
 */
interface BaseMessage {
  type: string;
  docId: string;
}

/** Client joining a document room */
interface JoinMessage extends BaseMessage {
  type: 'join';
  userId: string;
  userName: string;
  userColor: string;
}

/** Client leaving a document room */
interface LeaveMessage extends BaseMessage {
  type: 'leave';
  userId: string;
}

/** Document state sync (CRDT state) */
interface SyncMessage extends BaseMessage {
  type: 'sync';
  userId: string;
  state: string; // Serialized CRDT state
}

/** Request full state from peers */
interface SyncRequestMessage extends BaseMessage {
  type: 'sync_request';
  userId: string;
}

/** Cursor/selection presence update */
interface PresenceMessage extends BaseMessage {
  type: 'presence';
  userId: string;
  userName: string;
  userColor: string;
  cursor: number | null;
  selectionStart: number | null;
  selectionEnd: number | null;
}

type ClientMessage = JoinMessage | LeaveMessage | SyncMessage | SyncRequestMessage | PresenceMessage;

/** Connected client metadata */
interface ConnectedClient {
  ws: WebSocket;
  docId: string;
  userId: string;
  userName: string;
  userColor: string;
  joinedAt: Date;
}

// =============================================================================
// State Management
// =============================================================================

/** Map of docId -> Set of connected clients */
const rooms = new Map<string, Set<ConnectedClient>>();

/** Map of WebSocket -> client metadata */
const clients = new Map<WebSocket, ConnectedClient>();

/** Server statistics */
const stats = {
  totalConnections: 0,
  totalMessages: 0,
  startTime: new Date(),
};

// =============================================================================
// Room Management
// =============================================================================

/**
 * Add a client to a document room
 */
function joinRoom(client: ConnectedClient): void {
  let room = rooms.get(client.docId);
  if (!room) {
    room = new Set();
    rooms.set(client.docId, room);
  }
  room.add(client);
  clients.set(client.ws, client);
  
  console.log(`[Room ${client.docId}] ${client.userName} joined (${room.size} clients)`);
}

/**
 * Remove a client from their room
 */
function leaveRoom(ws: WebSocket): ConnectedClient | undefined {
  const client = clients.get(ws);
  if (!client) return undefined;
  
  const room = rooms.get(client.docId);
  if (room) {
    room.delete(client);
    if (room.size === 0) {
      rooms.delete(client.docId);
    }
    console.log(`[Room ${client.docId}] ${client.userName} left (${room.size} clients remaining)`);
  }
  
  clients.delete(ws);
  return client;
}

/**
 * Broadcast a message to all clients in a room except the sender
 */
function broadcast(docId: string, message: object, excludeWs?: WebSocket): void {
  const room = rooms.get(docId);
  if (!room) return;
  
  const payload = JSON.stringify(message);
  
  for (const client of room) {
    if (client.ws !== excludeWs && client.ws.readyState === WebSocket.OPEN) {
      client.ws.send(payload);
    }
  }
}

/**
 * Get list of users in a room (for presence)
 */
function getRoomUsers(docId: string): Array<{ userId: string; userName: string; userColor: string }> {
  const room = rooms.get(docId);
  if (!room) return [];
  
  return Array.from(room).map(client => ({
    userId: client.userId,
    userName: client.userName,
    userColor: client.userColor,
  }));
}

// =============================================================================
// WebSocket Message Handlers
// =============================================================================

/**
 * Handle incoming WebSocket messages
 */
function handleMessage(ws: WebSocket, data: string): void {
  stats.totalMessages++;
  
  let message: ClientMessage;
  try {
    message = JSON.parse(data);
  } catch (error) {
    console.error('Invalid JSON message:', data);
    return;
  }
  
  switch (message.type) {
    case 'join':
      handleJoin(ws, message);
      break;
      
    case 'leave':
      handleLeave(ws);
      break;
      
    case 'sync':
      handleSync(ws, message);
      break;
      
    case 'sync_request':
      handleSyncRequest(ws, message);
      break;
      
    case 'presence':
      handlePresence(ws, message);
      break;
      
    default:
      console.warn('Unknown message type:', (message as BaseMessage).type);
  }
}

/**
 * Handle client joining a document room
 */
function handleJoin(ws: WebSocket, message: JoinMessage): void {
  const client: ConnectedClient = {
    ws,
    docId: message.docId,
    userId: message.userId,
    userName: message.userName,
    userColor: message.userColor,
    joinedAt: new Date(),
  };
  
  joinRoom(client);
  
  // Notify others that a new user joined
  broadcast(message.docId, {
    type: 'user_joined',
    userId: message.userId,
    userName: message.userName,
    userColor: message.userColor,
  }, ws);
  
  // Send current user list to the new client
  const users = getRoomUsers(message.docId).filter(u => u.userId !== message.userId);
  ws.send(JSON.stringify({
    type: 'room_users',
    users,
  }));
}

/**
 * Handle client leaving
 */
function handleLeave(ws: WebSocket): void {
  const client = leaveRoom(ws);
  
  if (client) {
    broadcast(client.docId, {
      type: 'user_left',
      userId: client.userId,
    });
  }
}

/**
 * Handle document state sync
 * Broadcasts the serialized CRDT state to all other clients
 */
function handleSync(ws: WebSocket, message: SyncMessage): void {
  const client = clients.get(ws);
  if (!client) return;
  
  // Relay state to all other clients in the room
  broadcast(message.docId, {
    type: 'sync',
    userId: message.userId,
    state: message.state,
  }, ws);
}

/**
 * Handle sync request (new client asking for current state)
 */
function handleSyncRequest(ws: WebSocket, message: SyncRequestMessage): void {
  const room = rooms.get(message.docId);
  if (!room || room.size <= 1) return;
  
  // Ask the first other client to send their state
  for (const client of room) {
    if (client.ws !== ws && client.ws.readyState === WebSocket.OPEN) {
      client.ws.send(JSON.stringify({
        type: 'sync_request',
        requesterId: message.userId,
      }));
      break; // Only ask one client
    }
  }
}

/**
 * Handle presence (cursor/selection) updates
 */
function handlePresence(ws: WebSocket, message: PresenceMessage): void {
  const client = clients.get(ws);
  if (!client) return;
  
  // Relay presence to all other clients
  broadcast(message.docId, {
    type: 'presence',
    userId: message.userId,
    userName: message.userName,
    userColor: message.userColor,
    cursor: message.cursor,
    selectionStart: message.selectionStart,
    selectionEnd: message.selectionEnd,
  }, ws);
}

// =============================================================================
// Express App & Swagger
// =============================================================================

const app = express();
app.use(cors({ origin: CORS_ORIGIN }));
app.use(express.json());

/**
 * Swagger/OpenAPI Configuration
 */
const swaggerOptions: swaggerJsdoc.Options = {
  definition: {
    openapi: '3.0.0',
    info: {
      title: 'Carnelia Sync Server API',
      version: '1.0.0',
      description: `
WebSocket relay server for Carnelia collaborative documents.

## WebSocket Protocol

Connect to \`ws://host:${PORT}/ws\` and send JSON messages:

### Message Types

#### join
Join a document room:
\`\`\`json
{
  "type": "join",
  "docId": "doc-123",
  "userId": "user-abc",
  "userName": "Alice",
  "userColor": "#FF6B6B"
}
\`\`\`

#### sync
Send document state to other clients:
\`\`\`json
{
  "type": "sync",
  "docId": "doc-123",
  "userId": "user-abc",
  "state": "<serialized CRDT state>"
}
\`\`\`

#### presence
Update cursor/selection position:
\`\`\`json
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
\`\`\`
      `,
      contact: {
        name: 'Carnelia',
        url: 'https://github.com/Agate-DB/Carnelia',
      },
    },
    servers: [
      {
        url: `http://localhost:${PORT}`,
        description: 'Development server',
      },
    ],
  },
  apis: ['./src/index.ts'],
};

const swaggerSpec = swaggerJsdoc(swaggerOptions);
app.use('/docs', swaggerUi.serve, swaggerUi.setup(swaggerSpec));

/**
 * @openapi
 * /health:
 *   get:
 *     summary: Health check endpoint
 *     tags: [Health]
 *     responses:
 *       200:
 *         description: Server is healthy
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 status:
 *                   type: string
 *                   example: ok
 *                 uptime:
 *                   type: number
 *                   description: Uptime in seconds
 */
app.get('/health', (req, res) => {
  const uptime = (Date.now() - stats.startTime.getTime()) / 1000;
  res.json({ status: 'ok', uptime });
});

/**
 * @openapi
 * /stats:
 *   get:
 *     summary: Server statistics
 *     tags: [Stats]
 *     responses:
 *       200:
 *         description: Server statistics
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 activeRooms:
 *                   type: number
 *                 activeConnections:
 *                   type: number
 *                 totalConnections:
 *                   type: number
 *                 totalMessages:
 *                   type: number
 *                 uptime:
 *                   type: number
 */
app.get('/stats', (req, res) => {
  const uptime = (Date.now() - stats.startTime.getTime()) / 1000;
  res.json({
    activeRooms: rooms.size,
    activeConnections: clients.size,
    totalConnections: stats.totalConnections,
    totalMessages: stats.totalMessages,
    uptime,
  });
});

/**
 * @openapi
 * /rooms:
 *   get:
 *     summary: List active document rooms
 *     tags: [Rooms]
 *     responses:
 *       200:
 *         description: List of active rooms
 *         content:
 *           application/json:
 *             schema:
 *               type: array
 *               items:
 *                 type: object
 *                 properties:
 *                   docId:
 *                     type: string
 *                   userCount:
 *                     type: number
 *                   users:
 *                     type: array
 *                     items:
 *                       type: object
 *                       properties:
 *                         userId:
 *                           type: string
 *                         userName:
 *                           type: string
 */
app.get('/rooms', (req, res) => {
  const roomList = Array.from(rooms.entries()).map(([docId, clients]) => ({
    docId,
    userCount: clients.size,
    users: Array.from(clients).map(c => ({
      userId: c.userId,
      userName: c.userName,
    })),
  }));
  res.json(roomList);
});

/**
 * @openapi
 * /rooms/{docId}:
 *   get:
 *     summary: Get details of a specific room
 *     tags: [Rooms]
 *     parameters:
 *       - in: path
 *         name: docId
 *         required: true
 *         schema:
 *           type: string
 *         description: Document ID
 *     responses:
 *       200:
 *         description: Room details
 *       404:
 *         description: Room not found
 */
app.get('/rooms/:docId', (req, res) => {
  const room = rooms.get(req.params.docId);
  if (!room) {
    return res.status(404).json({ error: 'Room not found' });
  }
  
  res.json({
    docId: req.params.docId,
    userCount: room.size,
    users: Array.from(room).map(c => ({
      userId: c.userId,
      userName: c.userName,
      userColor: c.userColor,
      joinedAt: c.joinedAt,
    })),
  });
});

// =============================================================================
// HTTP & WebSocket Server
// =============================================================================

const server = createServer(app);

const wss = new WebSocketServer({ 
  server,
  path: '/ws',
});

wss.on('connection', (ws) => {
  stats.totalConnections++;
  console.log('New WebSocket connection');
  
  ws.on('message', (data) => {
    handleMessage(ws, data.toString());
  });
  
  ws.on('close', () => {
    handleLeave(ws);
  });
  
  ws.on('error', (error) => {
    console.error('WebSocket error:', error);
    handleLeave(ws);
  });
});

// =============================================================================
// Start Server
// =============================================================================

server.listen(PORT, () => {
  console.log(`
╔═══════════════════════════════════════════════════════════╗
║           Carnelia Sync Server                            ║
╠═══════════════════════════════════════════════════════════╣
║  HTTP API:    http://localhost:${PORT}                      ║
║  WebSocket:   ws://localhost:${PORT}/ws                     ║
║  Swagger UI:  http://localhost:${PORT}/docs                 ║
╚═══════════════════════════════════════════════════════════╝
  `);
});

export { app, server, wss };
