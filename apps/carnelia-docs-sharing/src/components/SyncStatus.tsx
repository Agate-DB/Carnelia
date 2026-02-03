/**
 * Sync Status Component
 * 
 * Shows connection status and sync information.
 */

interface SyncStatusProps {
  isConnected: boolean;
  isSyncing?: boolean;
  lastSyncTime?: Date | null;
  pendingChanges?: number;
  userCount?: number;
}

export function SyncStatus({ 
  isConnected, 
  isSyncing = false,
  lastSyncTime,
  pendingChanges = 0,
  userCount = 1,
}: SyncStatusProps) {
  const statusColor = isConnected 
    ? 'bg-green-500' 
    : 'bg-red-500';
  
  const statusText = isConnected
    ? isSyncing ? 'Syncing...' : 'Connected'
    : 'Offline';

  return (
    <div className="flex items-center gap-4 text-sm text-gray-600">
      {/* Connection status */}
      <div className="flex items-center gap-2">
        <div className={`w-2 h-2 rounded-full ${statusColor} ${isSyncing ? 'animate-pulse' : ''}`} />
        <span>{statusText}</span>
      </div>
      
      {/* User count */}
      <div className="flex items-center gap-1">
        <span>👥</span>
        <span>{userCount} {userCount === 1 ? 'user' : 'users'}</span>
      </div>
      
      {/* Pending changes */}
      {pendingChanges > 0 && (
        <div className="flex items-center gap-1 text-amber-600">
          <span>⏳</span>
          <span>{pendingChanges} pending</span>
        </div>
      )}
      
      {/* Last sync time */}
      {lastSyncTime && (
        <div className="text-gray-400">
          Last synced: {lastSyncTime.toLocaleTimeString()}
        </div>
      )}
    </div>
  );
}
