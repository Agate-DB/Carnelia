import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
  ],
  // Support WASM files
  optimizeDeps: {
    exclude: ['mdcs_wasm'],
  },
  server: {
    fs: {
      // Allow serving WASM files from src directory
      allow: ['..'],
    },
  },
})
