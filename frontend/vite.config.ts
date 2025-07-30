import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    {
      name: 'security-headers',
      configureServer(server) {
        server.middlewares.use((req, res, next) => {
          // Security headers
          res.setHeader('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');
          res.setHeader('X-Content-Type-Options', 'nosniff');
          res.setHeader('X-Frame-Options', 'DENY');
          res.setHeader('Content-Security-Policy', "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:; connect-src 'self' ws: wss: https: http://localhost:* http://127.0.0.1:*; font-src 'self' data:;");
          res.setHeader('X-XSS-Protection', '1; mode=block');
          res.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin');
          res.setHeader('Permissions-Policy', 'camera=(), microphone=(), geolocation=()');
          
          // Rate limiting headers (placeholder)
          res.setHeader('X-RateLimit-Limit', '1000');
          res.setHeader('X-RateLimit-Remaining', '999');
          res.setHeader('X-RateLimit-Reset', Math.floor(Date.now() / 1000) + 3600);
          
          next();
        });
      },
    },
  ],
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
    port: 4010,
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
