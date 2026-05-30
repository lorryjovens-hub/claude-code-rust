import path from 'path';
import fs from 'fs';
import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';

const host = process.env.TAURI_DEV_HOST;
const packageJson = JSON.parse(fs.readFileSync('./package.json', 'utf-8'));

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, '.', '');
  return {
    base: './',
    plugins: [react()],
    define: {
      __APP_VERSION__: JSON.stringify(packageJson.version),
    },
    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src'),
      },
    },
    clearScreen: false,
    server: {
      port: 5200,
      strictPort: true,
      host: '127.0.0.1',
      hmr: { protocol: 'ws', host: '127.0.0.1', port: 5201 },
      watch: { ignored: ['**/src-tauri/**'] },
    },
    envPrefix: ['VITE_', 'TAURI_ENV_*'],
    build: {
      outDir: 'dist',
      reportCompressedSize: false,
      target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari14',
      minify: mode === 'production' ? 'esbuild' : false,
      esbuild: {
        drop: mode === 'production' ? [] : [],
      },
      sourcemap: true,
      rollupOptions: {
        output: {
          manualChunks: {
            recharts: ['recharts'],
          },
        },
      },
    },
  };
});
