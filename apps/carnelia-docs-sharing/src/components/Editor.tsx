/**
 * Editor Component
 * 
 * Main collaborative text editor with CRDT-backed state
 * and WebSocket-based real-time sync.
 */

import { useRef, useCallback, useEffect, useState } from 'react';
import { Toolbar } from './Toolbar';
import { RemoteCursors, UserAvatars } from './RemoteCursors';
import { SyncStatus } from './SyncStatus';
import { useCollaborativeDocument } from '../hooks/useCollaborativeDocument';
import { useSync } from '../hooks/useSync';

// Sync server URL - override with environment variable in production
const SYNC_SERVER_URL = import.meta.env.VITE_SYNC_SERVER_URL || 'ws://localhost:3001/ws';

interface EditorProps {
  docId: string;
  userName?: string;
}

export function Editor({ docId, userName = 'Anonymous' }: EditorProps) {
  const editorRef = useRef<HTMLDivElement>(null);
  const [selection, setSelection] = useState<{ start: number; end: number } | null>(null);
  
  // CRDT document state
  const {
    text,
    html,
    isReady,
    version,
    insert,
    deleteText,
    applyBold,
    applyItalic,
    applyUnderline,
    applyStrikethrough,
    applyLink,
    updateCursor,
    updateSelection,
    userId,
    userColor,
    serialize,
    merge,
  } = useCollaborativeDocument({
    docId,
    userName,
  });
  
  // Logging helper
  const log = useCallback((category: string, message: string, data?: unknown) => {
    const timestamp = new Date().toISOString().split('T')[1].slice(0, 12);
    const style = {
      'INPUT': 'color: #22c55e; font-weight: bold',
      'SYNC': 'color: #3b82f6; font-weight: bold',
      'RECV': 'color: #f59e0b; font-weight: bold',
      'STATE': 'color: #8b5cf6; font-weight: bold',
      'CURSOR': 'color: #6b7280',
    }[category] || 'color: #888';
    
    if (data !== undefined) {
      console.log(`%c[${timestamp}] [${category}]%c ${message}`, style, 'color: inherit', data);
    } else {
      console.log(`%c[${timestamp}] [${category}]%c ${message}`, style, 'color: inherit');
    }
  }, []);
  
  // Log when document becomes ready
  useEffect(() => {
    if (isReady) {
      log('STATE', `Document ready: docId=${docId}, userId=${userId.slice(0,8)}..., version=${version}`);
    }
  }, [isReady, docId, userId, version, log]);
  
  // Stable callbacks for WebSocket sync
  const handleRemoteState = useCallback((state: string) => {
    log('RECV', `Received remote state (${state.length} bytes), merging...`);
    merge(state);
    log('STATE', `After merge: version=${version}, text length=${text.length}`);
  }, [merge, log, version, text.length]);
  
  const handleSyncRequested = useCallback(() => {
    const state = serialize();
    log('SYNC', `Peer requested sync, sending state (${state?.length || 0} bytes)`);
    return state;
  }, [serialize, log]);
  
  // WebSocket sync - only connect when document is ready with valid IDs
  const {
    isConnected,
    remoteUsers,
    sendState,
    sendPresence,
    error: syncError,
  } = useSync({
    serverUrl: SYNC_SERVER_URL,
    docId,
    userId,
    userName,
    userColor,
    onRemoteState: handleRemoteState,
    onSyncRequested: handleSyncRequested,
    debug: true,
    enabled: isReady && !!userId && !!userColor, // Only connect when ready
  });
  
  // Log connection state changes
  useEffect(() => {
    if (isReady) {
      log('SYNC', isConnected 
        ? `Connected to sync server (${remoteUsers.length} other users)` 
        : `Disconnected from sync server`);
    }
  }, [isConnected, isReady, remoteUsers.length, log]);
  
  // Send state to peers when local changes happen (debounced)
  const syncTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const syncToPeers = useCallback(() => {
    // Clear any pending sync
    if (syncTimeoutRef.current) {
      clearTimeout(syncTimeoutRef.current);
    }
    
    // Debounce: wait 150ms after last change before syncing
    syncTimeoutRef.current = setTimeout(() => {
      if (isConnected) {
        const state = serialize();
        if (state) {
          log('SYNC', `Sending state to peers (${state.length} bytes, version=${version})`);
          sendState(state);
        }
      }
    }, 150);
  }, [isConnected, serialize, sendState, log, version]);
  
  // Cleanup sync timeout on unmount
  useEffect(() => {
    return () => {
      if (syncTimeoutRef.current) {
        clearTimeout(syncTimeoutRef.current);
      }
    };
  }, []);
  
  // Sync presence when cursor/selection changes
  const syncPresence = useCallback((start: number, end: number) => {
    if (isConnected) {
      if (start === end) {
        sendPresence(start, null, null);
      } else {
        sendPresence(null, start, end);
      }
    }
  }, [isConnected, sendPresence]);

  // Track selection changes
  const handleSelectionChange = useCallback(() => {
    const sel = window.getSelection();
    if (!sel || !editorRef.current) return;
    
    const range = sel.getRangeAt(0);
    const preSelectionRange = document.createRange();
    preSelectionRange.selectNodeContents(editorRef.current);
    preSelectionRange.setEnd(range.startContainer, range.startOffset);
    const start = preSelectionRange.toString().length;
    const end = start + range.toString().length;
    
    setSelection({ start, end });
    
    if (start === end) {
      updateCursor(start);
    } else {
      updateSelection(start, end);
    }
    
    // Sync cursor/selection to peers
    syncPresence(start, end);
  }, [updateCursor, updateSelection, syncPresence]);

  // Get cursor position in contentEditable
  const getCursorPosition = useCallback((): number => {
    const sel = window.getSelection();
    if (!sel || !sel.rangeCount || !editorRef.current) return 0;
    
    const range = sel.getRangeAt(0);
    const preCaretRange = range.cloneRange();
    preCaretRange.selectNodeContents(editorRef.current);
    preCaretRange.setEnd(range.startContainer, range.startOffset);
    return preCaretRange.toString().length;
  }, []);
  
  // Set cursor position in contentEditable
  const setCursorPosition = useCallback((position: number) => {
    const editor = editorRef.current;
    if (!editor) return;
    
    const sel = window.getSelection();
    if (!sel) return;
    
    // Walk through text nodes to find the right position
    const walker = document.createTreeWalker(editor, NodeFilter.SHOW_TEXT, null);
    let currentPos = 0;
    let node: Text | null = null;
    
    while ((node = walker.nextNode() as Text | null)) {
      const nodeLength = node.length;
      if (currentPos + nodeLength >= position) {
        const range = document.createRange();
        range.setStart(node, position - currentPos);
        range.collapse(true);
        sel.removeAllRanges();
        sel.addRange(range);
        return;
      }
      currentPos += nodeLength;
    }
    
    // If position is beyond text, place at end
    const range = document.createRange();
    range.selectNodeContents(editor);
    range.collapse(false);
    sel.removeAllRanges();
    sel.addRange(range);
  }, []);

  // Handle beforeinput for proper text insertion
  const handleBeforeInput = useCallback((e: InputEvent) => {
    e.preventDefault();
    
    const cursorPos = getCursorPosition();
    
    switch (e.inputType) {
      case 'insertText':
      case 'insertCompositionText':
        if (e.data) {
          log('INPUT', `Insert "${e.data}" at position ${cursorPos}`);
          insert(cursorPos, e.data);
          syncToPeers();
          // Cursor will be repositioned after React re-render
          requestAnimationFrame(() => {
            setCursorPosition(cursorPos + e.data!.length);
          });
        }
        break;
        
      case 'insertParagraph':
      case 'insertLineBreak':
        log('INPUT', `Insert newline at position ${cursorPos}`);
        insert(cursorPos, '\n');
        syncToPeers();
        requestAnimationFrame(() => {
          setCursorPosition(cursorPos + 1);
        });
        break;
        
      case 'deleteContentBackward':
        if (cursorPos > 0) {
          log('INPUT', `Delete backward at position ${cursorPos}`);
          deleteText(cursorPos - 1, 1);
          syncToPeers();
          requestAnimationFrame(() => {
            setCursorPosition(cursorPos - 1);
          });
        }
        break;
        
      case 'deleteContentForward':
        if (cursorPos < text.length) {
          log('INPUT', `Delete forward at position ${cursorPos}`);
          deleteText(cursorPos, 1);
          syncToPeers();
          requestAnimationFrame(() => {
            setCursorPosition(cursorPos);
          });
        }
        break;
        
      case 'deleteByCut':
      case 'deleteByDrag':
      case 'deleteContent':
        // Handle selection deletion
        if (selection && selection.start !== selection.end) {
          const deleteLength = selection.end - selection.start;
          log('INPUT', `Delete selection [${selection.start}-${selection.end}] (${deleteLength} chars)`);
          deleteText(selection.start, deleteLength);
          syncToPeers();
          requestAnimationFrame(() => {
            setCursorPosition(selection.start);
          });
        }
        break;
        
      case 'insertFromPaste':
        if (e.data) {
          // Handle selection replacement on paste
          if (selection && selection.start !== selection.end) {
            log('INPUT', `Paste "${e.data.slice(0, 20)}${e.data.length > 20 ? '...' : ''}" replacing selection [${selection.start}-${selection.end}]`);
            deleteText(selection.start, selection.end - selection.start);
            insert(selection.start, e.data);
            syncToPeers();
            requestAnimationFrame(() => {
              setCursorPosition(selection.start + e.data!.length);
            });
          } else {
            log('INPUT', `Paste "${e.data.slice(0, 20)}${e.data.length > 20 ? '...' : ''}" at position ${cursorPos}`);
            insert(cursorPos, e.data);
            syncToPeers();
            requestAnimationFrame(() => {
              setCursorPosition(cursorPos + e.data!.length);
            });
          }
        }
        break;
        
      default:
        log('INPUT', `Unhandled input type: ${e.inputType}`);
    }
  }, [getCursorPosition, setCursorPosition, insert, deleteText, text.length, selection, syncToPeers, log]);
  
  // Handle paste event to get clipboard data
  const handlePaste = useCallback((e: React.ClipboardEvent) => {
    e.preventDefault();
    const pastedText = e.clipboardData.getData('text/plain');
    if (pastedText) {
      const cursorPos = getCursorPosition();
      log('INPUT', `Paste from clipboard: "${pastedText.slice(0, 30)}${pastedText.length > 30 ? '...' : ''}" (${pastedText.length} chars)`);
      
      // Handle selection replacement
      if (selection && selection.start !== selection.end) {
        deleteText(selection.start, selection.end - selection.start);
        insert(selection.start, pastedText);
        syncToPeers();
        requestAnimationFrame(() => {
          setCursorPosition(selection.start + pastedText.length);
        });
      } else {
        insert(cursorPos, pastedText);
        syncToPeers();
        requestAnimationFrame(() => {
          setCursorPosition(cursorPos + pastedText.length);
        });
      }
    }
  }, [getCursorPosition, setCursorPosition, insert, deleteText, selection, syncToPeers, log]);
  
  // Attach beforeinput listener (React doesn't have native support)
  useEffect(() => {
    const editor = editorRef.current;
    if (!editor) return;
    
    const handler = (e: Event) => handleBeforeInput(e as InputEvent);
    editor.addEventListener('beforeinput', handler);
    
    return () => {
      editor.removeEventListener('beforeinput', handler);
    };
  }, [handleBeforeInput]);

  // Toolbar handlers
  const handleBold = useCallback(() => {
    if (selection && selection.start !== selection.end) {
      log('INPUT', `Apply BOLD to selection [${selection.start}-${selection.end}]`);
      applyBold(selection.start, selection.end);
      syncToPeers();
    }
  }, [selection, applyBold, syncToPeers, log]);

  const handleItalic = useCallback(() => {
    if (selection && selection.start !== selection.end) {
      log('INPUT', `Apply ITALIC to selection [${selection.start}-${selection.end}]`);
      applyItalic(selection.start, selection.end);
      syncToPeers();
    }
  }, [selection, applyItalic, syncToPeers, log]);

  const handleUnderline = useCallback(() => {
    if (selection && selection.start !== selection.end) {
      log('INPUT', `Apply UNDERLINE to selection [${selection.start}-${selection.end}]`);
      applyUnderline(selection.start, selection.end);
      syncToPeers();
    }
  }, [selection, applyUnderline, syncToPeers, log]);

  const handleStrikethrough = useCallback(() => {
    if (selection && selection.start !== selection.end) {
      log('INPUT', `Apply STRIKETHROUGH to selection [${selection.start}-${selection.end}]`);
      applyStrikethrough(selection.start, selection.end);
      syncToPeers();
    }
  }, [selection, applyStrikethrough, syncToPeers, log]);

  const handleLinkInsert = useCallback(() => {
    if (selection && selection.start !== selection.end) {
      const url = prompt('Enter URL:');
      if (url) {
        log('INPUT', `Apply LINK "${url}" to selection [${selection.start}-${selection.end}]`);
        applyLink(selection.start, selection.end, url);
        syncToPeers();
      }
    }
  }, [selection, applyLink, syncToPeers, log]);

  // Set up selection listener
  useEffect(() => {
    document.addEventListener('selectionchange', handleSelectionChange);
    return () => {
      document.removeEventListener('selectionchange', handleSelectionChange);
    };
  }, [handleSelectionChange]);

  if (!isReady) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex items-center gap-3">
          <div className="w-6 h-6 border-2 border-blue-500 border-t-transparent rounded-full animate-spin" />
          <span className="text-gray-600">Loading editor...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full relative">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <div>
          <h2 className="text-lg font-semibold text-gray-800">
            {docId}
          </h2>
          <p className="text-sm text-gray-500">
            Version {version} • {text.length} characters
            {syncError && <span className="text-red-500 ml-2">• {syncError}</span>}
          </p>
        </div>
        
        <div className="flex items-center gap-4">
          <UserAvatars 
            users={remoteUsers} 
            localUser={{ name: userName, color: userColor }} 
          />
          <SyncStatus 
            isConnected={isConnected}
            userCount={remoteUsers.length + 1}
          />
        </div>
      </div>
      
      {/* Toolbar */}
      <Toolbar
        onBold={handleBold}
        onItalic={handleItalic}
        onUnderline={handleUnderline}
        onStrikethrough={handleStrikethrough}
        onLink={handleLinkInsert}
        disabled={!selection || selection.start === selection.end}
      />
      
      {/* Editor area */}
      <div className="flex-1 relative my-12 mx-12 md:mx-16 lg:mx-24 xl:mx-32 border border-gray-200 shadow-xl shadow-bg-gray-400  min-h-screen overflow-none">
        <RemoteCursors users={remoteUsers} textLength={text.length} />
        
        <div
          ref={editorRef}
          className="min-h-full p-6 outline-none prose prose-sm max-w-none whitespace-pre-wrap"
          contentEditable
          suppressContentEditableWarning
          onPaste={handlePaste}
          dangerouslySetInnerHTML={{ __html: html }}
        />
      </div>
      
      {/* Footer */}
      <div className="flex fixed bottom-0 items-center justify-between px-4 py-2 bg-gray-50 border-t border-gray-200 text-xs text-gray-500">
        <span>
          User ID: {userId.slice(0, 8)}...
        </span>
        <span>
          Powered by Carnelia CRDT
        </span>
      </div>
    </div>
  );
}
