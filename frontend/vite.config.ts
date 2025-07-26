import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    host: "0.0.0.0",
    port: 4001,
    watch: {
      usePolling: true,
    },
  },
  preview: {
    port: 4001,
    host: "0.0.0.0",
  },
});
