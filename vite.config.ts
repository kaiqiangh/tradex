import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { integrationBridge } from './tests/integration-bridge.ts';

export default defineConfig(({ mode }) => ({
  plugins: [react(), ...(mode === 'integration' ? [integrationBridge()] : [])],
  server: { port: 1420, strictPort: true, host: '127.0.0.1' },
  clearScreen: false,
}));
