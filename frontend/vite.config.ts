import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    // In Docker the backend service is reachable by container name on the
    // internal network. Outside Docker it falls back to localhost.
    proxy: {
      // Use /api/ (with trailing slash) so that /api-keys (a frontend route)
      // is not accidentally proxied to the backend.
      '/api/': {
        target: process.env.VITE_API_URL || 'http://localhost:3000',
        changeOrigin: true,
      },
      '/ws': {
        target: process.env.VITE_API_URL || 'http://localhost:3000',
        ws: true,
      },
      '/llms.txt': {
        target: process.env.VITE_API_URL || 'http://localhost:3000',
        changeOrigin: true,
      },
      '/llms-full.txt': {
        target: process.env.VITE_API_URL || 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
})
