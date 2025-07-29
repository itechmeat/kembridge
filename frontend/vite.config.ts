import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  define: {
    global: "globalThis",
    "process.env": {},
  },
  resolve: {
    alias: {
      buffer: "buffer",
      process: "process/browser",
    },
  },
  server: {
    host: "0.0.0.0",
    port: 4100,
    watch: {
      usePolling: true,
    },
  },
  preview: {
    port: 4001,
    host: "0.0.0.0",
  },
  optimizeDeps: {
    include: ["react", "react-dom", "buffer", "process"],
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ["react", "react-dom"],
          wallet: [
            "@near-wallet-selector/core",
            "@rainbow-me/rainbowkit",
            "wagmi",
          ],
        },
      },
    },
  },
});
