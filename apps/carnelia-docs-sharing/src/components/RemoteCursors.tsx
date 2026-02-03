/**
 * Remote Cursors Component
 * 
 * Renders cursor indicators for remote collaborators.
 */

import type { SyncUser } from '../hooks/useSync';

// Accept either the SyncUser type (from useSync) or the legacy RemoteUser type
interface RemoteCursorsUser {
  userId?: string;
  id?: string;
  userName?: string;
  name?: string;
  userColor?: string;
  color?: string;
  cursor: number | null;
  selectionStart: number | null;
  selectionEnd: number | null;
}

interface RemoteCursorsProps {
  users: RemoteCursorsUser[] | SyncUser[];
  textLength: number;
}

// Helper to normalize user object
function normalizeUser(user: RemoteCursorsUser) {
  return {
    id: user.userId || user.id || 'unknown',
    name: user.userName || user.name || 'Anonymous',
    color: user.userColor || user.color || '#888888',
    cursor: user.cursor,
    selectionStart: user.selectionStart,
    selectionEnd: user.selectionEnd,
  };
}

export function RemoteCursors({ users, textLength }: RemoteCursorsProps) {
  if (users.length === 0) return null;

  return (
    <div className="absolute inset-0 pointer-events-none overflow-hidden">
      {users.map((rawUser) => {
        const user = normalizeUser(rawUser as RemoteCursorsUser);
        if (user.cursor === null) return null;
        
        // Calculate cursor position as percentage
        const position = textLength > 0 
          ? (user.cursor / textLength) * 100 
          : 0;
        
        return (
          <div
            key={user.id}
            className="absolute top-0 h-full transition-all duration-150"
            style={{ 
              left: `${Math.min(position, 100)}%`,
            }}
          >
            {/* Cursor line */}
            <div 
              className="w-0.5 h-full"
              style={{ backgroundColor: user.color }}
            />
            
            {/* User label */}
            <div 
              className="absolute -top-6 left-0 px-2 py-0.5 rounded text-xs text-white whitespace-nowrap"
              style={{ backgroundColor: user.color }}
            >
              {user.name}
            </div>
            
            {/* Selection highlight */}
            {user.selectionStart !== null && user.selectionEnd !== null && (
              <div
                className="absolute top-0 h-full opacity-20"
                style={{
                  backgroundColor: user.color,
                  left: `${(user.selectionStart / textLength) * 100}%`,
                  width: `${((user.selectionEnd - user.selectionStart) / textLength) * 100}%`,
                }}
              />
            )}
          </div>
        );
      })}
    </div>
  );
}

/**
 * User Avatar List Component
 * 
 * Shows avatars of all connected users.
 */
interface UserAvatarsProps {
  users: RemoteCursorsUser[] | SyncUser[];
  localUser: {
    name: string;
    color: string;
  };
}

export function UserAvatars({ users, localUser }: UserAvatarsProps) {
  const allUsers = [
    { id: 'local', name: localUser.name, color: localUser.color, isLocal: true },
    ...users.map(u => {
      const normalized = normalizeUser(u as RemoteCursorsUser);
      return { ...normalized, isLocal: false };
    }),
  ];

  return (
    <div className="flex items-center -space-x-2">
      {allUsers.map((user, index) => (
        <div
          key={user.id}
          className={`
            w-8 h-8 rounded-full flex items-center justify-center
            text-white text-sm font-medium
            border-2 border-white
            ${user.isLocal ? 'ring-2 ring-blue-400' : ''}
          `}
          style={{ 
            backgroundColor: user.color,
            zIndex: allUsers.length - index,
          }}
          title={user.name + (user.isLocal ? ' (you)' : '')}
        >
          {user.name.charAt(0).toUpperCase()}
        </div>
      ))}
    </div>
  );
}
