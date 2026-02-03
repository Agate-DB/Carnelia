/**
 * Hooks index
 * 
 * Re-exports all custom hooks for easy importing.
 */

export { useCollaborativeDocument } from './useCollaborativeDocument';
export type { 
  RemoteUser, 
  UseCollaborativeDocumentOptions, 
  UseCollaborativeDocumentReturn 
} from './useCollaborativeDocument';

export { useSync } from './useSync';
export type { 
  SyncUser, 
  UseSyncOptions, 
  UseSyncReturn 
} from './useSync';
