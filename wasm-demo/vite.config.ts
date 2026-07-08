import { defineConfig } from "vite";

import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), tailwindcss()],
  base: "",
  build: {
    outDir: "build",
    sourcemap: true,
  },
  optimizeDeps: {
    // Keep the wasm-pack package out of esbuild pre-bundling so the
    // `new URL("acs_bg.wasm", import.meta.url)` lookup keeps working.
    exclude: ["acs"],
  },
  server: {
    fs: {
      allow: [".."],
    },
    allowedHosts: [
      "localhost",
      "127.0.0.1",
      "af20-2804-7f0-6942-b6cf-514-2588-c4ee-1e8c.ngrok-free.app",
    ],
  },
});
