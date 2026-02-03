/**
 * Carnelia WASM Client
 * 
 * Wrapper around the WASM bindings for easier use in React.
 */

import init, { 
  CollaborativeDocument, 
  UserPresence,
  generate_replica_id,
  generate_user_color,
} from '../wasm/mdcs_wasm';

let wasmInitialized = false;
let initPromise: Promise<void> | null = null;

/**
 * Initialize the WASM module. Safe to call multiple times.
 */
export async function initCarnelia(): Promise<void> {
  if (wasmInitialized) return;
  
  if (!initPromise) {
    initPromise = init().then(() => {
      wasmInitialized = true;
    });
  }
  
  await initPromise;
}

/**
 * Check if WASM is initialized
 */
export function isCarmeliaReady(): boolean {
  return wasmInitialized;
}

// Re-export WASM types
export { CollaborativeDocument, UserPresence };

// Re-export utility functions
export { generate_replica_id as generateReplicaId, generate_user_color as generateUserColor };

/**
 * Generate a unique document ID
 */
export function generateDocId(): string {
  return `doc-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

/**
 * Generate a unique user ID
 */
export function generateUserId(): string {
  return `user-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

// Predefined color palette for users
export const USER_COLORS = [
  '#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4',
  '#FFEAA7', '#DDA0DD', '#98D8C8', '#F7DC6F',
  '#E74C3C', '#3498DB', '#2ECC71', '#9B59B6',
  '#1ABC9C', '#F39C12', '#E91E63', '#00BCD4',
];

/**
 * Get a color from the palette based on user index
 */
export function getUserColor(index: number): string {
  return USER_COLORS[index % USER_COLORS.length];
}
