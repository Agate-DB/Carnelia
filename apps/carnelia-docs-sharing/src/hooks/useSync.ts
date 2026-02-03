/**
 * useSync Hook
 * 
 * Manages WebSocket connection to the Carnelia sync server
 * for real-time document collaboration.
 */

import { useEffect, useRef, useCallback, useState } from 'react';

// =============================================================================
// Types
// =============================================================================

export interface SyncUser {
  userId: string;
  userName: string;
  userColor: string;
  cursor: number | null;
  selectionStart: number | null;
  selectionEnd: number | null;
}

export interface UseSyncOptions {
  /** WebSocket server URL (e.g., ws://localhost:3001/ws) */
  serverUrl: string;
  /** Document ID to sync */
  docId: string;
  /** Current user's ID */
  userId: string;
  /** Current user's display name */
  userName: string;
  /** Current user's assigned color */
  userColor: string;
  /** Callback when remote state is received */
  onRemoteState: (state: string) => void;
  /** Callback when a user joins */
  onUserJoined?: (user: SyncUser) => void;
  /** Callback when a user leaves */
  onUserLeft?: (userId: string) => void;
  /** Callback when remote presence updates */
  onPresenceUpdate?: (user: SyncUser) => void;
  /** Callback when sync is requested by new peer */
  onSyncRequested?: () => string | null;
  /** Enable debug logging */
  debug?: boolean;
  /** Enable/disable the connection (default: true) */
  enabled?: boolean;
}

export interface UseSyncReturn {
  /** Is connected to the server */
  isConnected: boolean;
  /** List of remote users in the room */
  remoteUsers: SyncUser[];
  /** Send local document state to peers */
  sendState: (state: string) => void;
  /** Send cursor/selection presence */
  sendPresence: (cursor: number | null, selectionStart: number | null, selectionEnd: number | null) => void;
  /** Request state from peers (for initial sync) */
  requestSync: () => void;
  /** Manually reconnect */
  reconnect: () => void;
  /** Connection error if any */
  error: string | null;
}

// =============================================================================
// Hook Implementation
// =============================================================================

export function useSync(options: UseSyncOptions): UseSyncReturn {
  const {
    serverUrl,
    docId,
    userId,
    userName,
    userColor,
    onRemoteState,
    onUserJoined,
    onUserLeft,
    onPresenceUpdate,
    onSyncRequested,
    debug = false,
    enabled = true,
  } = options;
  
  const [isConnected, setIsConnected] = useState(false);
  const [remoteUsers, setRemoteUsers] = useState<SyncUser[]>([]);
  const [error, setError] = useState<string | null>(null);
  
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const reconnectAttempts = useRef(0);
  const isCleaningUp = useRef(false);
  const hasConnected = useRef(false); // Guard against double-mount in StrictMode
  
  // Store callbacks in refs to avoid dependency issues
  const callbacksRef = useRef({
    onRemoteState,
    onUserJoined,
    onUserLeft,
    onPresenceUpdate,
    onSyncRequested,
  });
  
  // Update callbacks ref when they change
  useEffect(() => {
    callbacksRef.current = {
      onRemoteState,
      onUserJoined,
      onUserLeft,
      onPresenceUpdate,
      onSyncRequested,
    };
  }, [onRemoteState, onUserJoined, onUserLeft, onPresenceUpdate, onSyncRequested]);
  
  // Store connection params in ref
  const paramsRef = useRef({ serverUrl, docId, userId, userName, userColor, debug, enabled });
  useEffect(() => {
    paramsRef.current = { serverUrl, docId, userId, userName, userColor, debug, enabled };
  }, [serverUrl, docId, userId, userName, userColor, debug, enabled]);
  
  const log = useCallback((...args: unknown[]) => {
    if (paramsRef.current.debug) {
      console.log('[useSync]', ...args);
    }
  }, []);
  
  // Connect to WebSocket server - stable function that reads from refs
  const connect = useCallback(() => {
    // Don't connect if disabled
    if (!paramsRef.current.enabled) {
      log('Skipping connect - disabled');
      return;
    }
    
    // Don't reconnect if we're cleaning up
    if (isCleaningUp.current) {
      log('Skipping connect - cleaning up');
      return;
    }
    
    // Don't create multiple connections
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      log('Skipping connect - already connected');
      return;
    }
    
    if (wsRef.current && wsRef.current.readyState === WebSocket.CONNECTING) {
      log('Skipping connect - connection in progress');
      return;
    }
    
    // Clean up any existing connection that's closing
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
    
    const { serverUrl, docId, userId, userName, userColor } = paramsRef.current;
    
    log('Connecting to', serverUrl);
    
    try {
      const ws = new WebSocket(serverUrl);
      wsRef.current = ws;
      
      ws.onopen = () => {
        log('Connected');
        setIsConnected(true);
        setError(null);
        reconnectAttempts.current = 0;
        
        // Join the room
        ws.send(JSON.stringify({
          type: 'join',
          docId,
          userId,
          userName,
          userColor,
        }));
        
        // Request current state from peers
        setTimeout(() => {
          if (ws.readyState === WebSocket.OPEN) {
            ws.send(JSON.stringify({
              type: 'sync_request',
              docId,
              userId,
            }));
          }
        }, 100);
      };
      
      ws.onmessage = (event) => {
        try {
          const message = JSON.parse(event.data);
          log('Received:', message);
          
          const callbacks = callbacksRef.current;
          
          switch (message.type) {
            case 'room_users':
              // Initial list of users in the room
              setRemoteUsers(message.users.map((u: { userId: string; userName: string; userColor: string }) => ({
                ...u,
                cursor: null,
                selectionStart: null,
                selectionEnd: null,
              })));
              break;
              
            case 'user_joined':
              // New user joined
              setRemoteUsers(prev => {
                if (prev.some(u => u.userId === message.userId)) {
                  return prev;
                }
                return [...prev, {
                  userId: message.userId,
                  userName: message.userName,
                  userColor: message.userColor,
                  cursor: null,
                  selectionStart: null,
                  selectionEnd: null,
                }];
              });
              callbacks.onUserJoined?.({
                userId: message.userId,
                userName: message.userName,
                userColor: message.userColor,
                cursor: null,
                selectionStart: null,
                selectionEnd: null,
              });
              break;
              
            case 'user_left':
              // User left
              setRemoteUsers(prev => prev.filter(u => u.userId !== message.userId));
              callbacks.onUserLeft?.(message.userId);
              break;
              
            case 'sync':
              // Remote state received
              callbacks.onRemoteState(message.state);
              break;
              
            case 'sync_request':
              // Peer requesting our state
              if (callbacks.onSyncRequested) {
                const state = callbacks.onSyncRequested();
                if (state && ws.readyState === WebSocket.OPEN) {
                  const params = paramsRef.current;
                  ws.send(JSON.stringify({
                    type: 'sync',
                    docId: params.docId,
                    userId: params.userId,
                    state,
                  }));
                }
              }
              break;
              
            case 'presence':
              // Remote presence update
              setRemoteUsers(prev => prev.map(u => 
                u.userId === message.userId
                  ? {
                      ...u,
                      cursor: message.cursor,
                      selectionStart: message.selectionStart,
                      selectionEnd: message.selectionEnd,
                    }
                  : u
              ));
              callbacks.onPresenceUpdate?.({
                userId: message.userId,
                userName: message.userName,
                userColor: message.userColor,
                cursor: message.cursor,
                selectionStart: message.selectionStart,
                selectionEnd: message.selectionEnd,
              });
              break;
          }
        } catch (err) {
          console.error('Failed to parse message:', err);
        }
      };
      
      ws.onclose = () => {
        log('Disconnected');
        setIsConnected(false);
        wsRef.current = null;
        
        // Only attempt to reconnect if not cleaning up
        if (!isCleaningUp.current) {
          const delay = Math.min(1000 * Math.pow(2, reconnectAttempts.current), 30000);
          reconnectAttempts.current++;
          
          log(`Reconnecting in ${delay}ms (attempt ${reconnectAttempts.current})`);
          reconnectTimeoutRef.current = setTimeout(connect, delay);
        }
      };
      
      ws.onerror = (err) => {
        console.error('WebSocket error:', err);
        setError('Connection error');
      };
      
    } catch (err) {
      console.error('Failed to connect:', err);
      setError('Failed to connect');
    }
  }, [log]); // Only depends on log which is stable
  
  // Connect on mount - only run once (if enabled)
  useEffect(() => {
    // Guard against React StrictMode double-mount
    if (hasConnected.current) {
      log('Skipping mount connect - already connected in this session');
      return;
    }
    
    // Only connect if enabled
    if (!paramsRef.current.enabled) {
      log('Skipping mount connect - disabled');
      return;
    }
    
    hasConnected.current = true;
    isCleaningUp.current = false;
    connect();
    
    return () => {
      isCleaningUp.current = true;
      hasConnected.current = false; // Reset on unmount so remount works
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
        reconnectTimeoutRef.current = null;
      }
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
      }
    };
  }, []); // Empty deps - only run on mount/unmount
  
  // Connect when enabled becomes true, disconnect when false
  useEffect(() => {
    if (enabled) {
      // Only connect if not already connected
      if (!wsRef.current || wsRef.current.readyState === WebSocket.CLOSED) {
        hasConnected.current = false; // Reset so connect() will work
        isCleaningUp.current = false;
        connect();
      }
    } else {
      // Disconnect when disabled
      if (wsRef.current) {
        wsRef.current.close();
        wsRef.current = null;
        setIsConnected(false);
      }
    }
  }, [enabled, connect]);
  
  // Reconnect if critical params change (serverUrl, docId)
  useEffect(() => {
    // Skip initial mount (handled by the effect above)
    if (!wsRef.current && !reconnectTimeoutRef.current) {
      return;
    }
    
    // Reconnect with new params
    isCleaningUp.current = false;
    reconnectAttempts.current = 0;
    connect();
  }, [serverUrl, docId, connect]); // Only reconnect if server or doc changes
  
  // Send document state to peers
  const sendState = useCallback((state: string) => {
    const ws = wsRef.current;
    const params = paramsRef.current;
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({
        type: 'sync',
        docId: params.docId,
        userId: params.userId,
        state,
      }));
    }
  }, []);
  
  // Send presence (cursor/selection)
  const sendPresence = useCallback((
    cursor: number | null,
    selectionStart: number | null,
    selectionEnd: number | null
  ) => {
    const ws = wsRef.current;
    const params = paramsRef.current;
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({
        type: 'presence',
        docId: params.docId,
        userId: params.userId,
        userName: params.userName,
        userColor: params.userColor,
        cursor,
        selectionStart,
        selectionEnd,
      }));
    }
  }, []);
  
  // Request sync from peers
  const requestSync = useCallback(() => {
    const ws = wsRef.current;
    const params = paramsRef.current;
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({
        type: 'sync_request',
        docId: params.docId,
        userId: params.userId,
      }));
    }
  }, []);
  
  // Manual reconnect
  const reconnect = useCallback(() => {
    isCleaningUp.current = false;
    reconnectAttempts.current = 0;
    connect();
  }, [connect]);
  
  return {
    isConnected,
    remoteUsers,
    sendState,
    sendPresence,
    requestSync,
    reconnect,
    error,
  };
}
