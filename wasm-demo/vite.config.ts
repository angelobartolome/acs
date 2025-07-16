import { defineConfig } from "vite";

import react from "@vitejs/plugin-react";

// https://vite.dev/config/
export default defineConfig({
  base: "",
  plugins: [react()],
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
});
