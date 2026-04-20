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
  assetsInclude: [
    "**/*.exr",
    "**/*.jpg",
    "**/*.jpeg",
    "**/*.svg",
    "**/*.hdr",
    "**/*.glb",
    "**/*.gltf",
    "**/*.woff2",
    "**/*.woff",
    "**/*.ttf",
  ],
  server: {
    fs: {
      allow: [".."],
    },
  },
});
