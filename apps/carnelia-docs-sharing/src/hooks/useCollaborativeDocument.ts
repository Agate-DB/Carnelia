/**
 * useCollaborativeDocument Hook
 * 
 * React hook for managing a collaborative document with CRDT-based
 * conflict resolution.
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { 
  initCarnelia, 
  CollaborativeDocument, 
  UserPresence,
  generateReplicaId,
  generateUserColor,
  generateUserId,
} from '../lib/carnelia-client';

export interface RemoteUser {
  id: string;
  name: string;
  color: string;
  cursor: number | null;
  selectionStart: number | null;
  selectionEnd: number | null;
}

export interface UseCollaborativeDocumentOptions {
  docId: string;
  userName?: string;
  onSync?: (state: string) => void;
  onRemoteChange?: () => void;
}

export interface UseCollaborativeDocumentReturn {
  // Document state
  text: string;
  html: string;
  isReady: boolean;
  version: number;
  
  // Document operations
  insert: (position: number, text: string) => void;
  deleteText: (position: number, length: number) => void;
  applyBold: (start: number, end: number) => void;
  applyItalic: (start: number, end: number) => void;
  applyUnderline: (start: number, end: number) => void;
  applyStrikethrough: (start: number, end: number) => void;
  applyLink: (start: number, end: number, url: string) => void;
  
  // Presence
  localPresence: UserPresence | null;
  remoteUsers: RemoteUser[];
  updateCursor: (position: number) => void;
  updateSelection: (start: number, end: number) => void;
  
  // Sync
  serialize: () => string | null;
  merge: (remoteState: string) => void;
  
  // User info
  userId: string;
  replicaId: string;
  userColor: string;
}

export function useCollaborativeDocument(
  options: UseCollaborativeDocumentOptions
): UseCollaborativeDocumentReturn {
  const { docId, userName = 'Anonymous', onSync, onRemoteChange } = options;
  
  // State
  const [isReady, setIsReady] = useState(false);
  const [text, setText] = useState('');
  const [html, setHtml] = useState('');
  const [version, setVersion] = useState(0);
  const [remoteUsers, setRemoteUsers] = useState<RemoteUser[]>([]);
  
  // Refs for stable values
  const docRef = useRef<CollaborativeDocument | null>(null);
  const presenceRef = useRef<UserPresence | null>(null);
  const userIdRef = useRef<string>(generateUserId());
  const replicaIdRef = useRef<string>('');
  const userColorRef = useRef<string>('');
  
  // Initialize WASM and create document
  useEffect(() => {
    let mounted = true;
    
    const initialize = async () => {
      try {
        await initCarnelia();
        
        if (!mounted) return;
        
        // Generate stable IDs
        replicaIdRef.current = generateReplicaId();
        userColorRef.current = generateUserColor();
        
        // Create document
        const doc = new CollaborativeDocument(docId, replicaIdRef.current);
        docRef.current = doc;
        
        // Create presence
        const presence = new UserPresence(
          userIdRef.current,
          userName,
          userColorRef.current
        );
        presenceRef.current = presence;
        
        // Update state
        setText(doc.get_text());
        setHtml(doc.get_html());
        setVersion(Number(doc.version()));
        setIsReady(true);
      } catch (error) {
        console.error('Failed to initialize collaborative document:', error);
      }
    };
    
    initialize();
    
    return () => {
      mounted = false;
      docRef.current = null;
      presenceRef.current = null;
    };
  }, [docId, userName]);
  
  // Helper to refresh state from document
  const refreshState = useCallback(() => {
    const doc = docRef.current;
    if (doc) {
      setText(doc.get_text());
      setHtml(doc.get_html());
      setVersion(Number(doc.version()));
      onSync?.(doc.serialize() ?? '');
    }
  }, [onSync]);
  
  // Document operations
  const insert = useCallback((position: number, insertText: string) => {
    const doc = docRef.current;
    if (doc) {
      doc.insert(position, insertText);
      refreshState();
    }
  }, [refreshState]);
  
  const deleteText = useCallback((position: number, length: number) => {
    const doc = docRef.current;
    if (doc) {
      doc.delete(position, length);
      refreshState();
    }
  }, [refreshState]);
  
  const applyBold = useCallback((start: number, end: number) => {
    const doc = docRef.current;
    if (doc) {
      doc.apply_bold(start, end);
      refreshState();
    }
  }, [refreshState]);
  
  const applyItalic = useCallback((start: number, end: number) => {
    const doc = docRef.current;
    if (doc) {
      doc.apply_italic(start, end);
      refreshState();
    }
  }, [refreshState]);
  
  const applyUnderline = useCallback((start: number, end: number) => {
    const doc = docRef.current;
    if (doc) {
      doc.apply_underline(start, end);
      refreshState();
    }
  }, [refreshState]);
  
  const applyStrikethrough = useCallback((start: number, end: number) => {
    const doc = docRef.current;
    if (doc) {
      doc.apply_strikethrough(start, end);
      refreshState();
    }
  }, [refreshState]);
  
  const applyLink = useCallback((start: number, end: number, url: string) => {
    const doc = docRef.current;
    if (doc) {
      doc.apply_link(start, end, url);
      refreshState();
    }
  }, [refreshState]);
  
  // Presence operations
  const updateCursor = useCallback((position: number) => {
    const presence = presenceRef.current;
    if (presence) {
      presence.set_cursor(position);
    }
  }, []);
  
  const updateSelection = useCallback((start: number, end: number) => {
    const presence = presenceRef.current;
    if (presence) {
      presence.set_selection(start, end);
    }
  }, []);
  
  // Sync operations
  const serialize = useCallback((): string | null => {
    const doc = docRef.current;
    if (doc) {
      try {
        return doc.serialize();
      } catch (error) {
        console.error('Serialization error:', error);
        return null;
      }
    }
    return null;
  }, []);
  
  const merge = useCallback((remoteState: string) => {
    const doc = docRef.current;
    if (doc) {
      try {
        doc.merge(remoteState);
        refreshState();
        onRemoteChange?.();
      } catch (error) {
        console.error('Merge error:', error);
      }
    }
  }, [refreshState, onRemoteChange]);
  
  return {
    // Document state
    text,
    html,
    isReady,
    version,
    
    // Document operations
    insert,
    deleteText,
    applyBold,
    applyItalic,
    applyUnderline,
    applyStrikethrough,
    applyLink,
    
    // Presence
    localPresence: presenceRef.current,
    remoteUsers,
    updateCursor,
    updateSelection,
    
    // Sync
    serialize,
    merge,
    
    // User info
    userId: userIdRef.current,
    replicaId: replicaIdRef.current,
    userColor: userColorRef.current,
  };
}
