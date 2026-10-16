// vite.config.js
import { defineConfig } from "vite";

export default defineConfig({
  build: {
    outDir: "../server/dist", // embedded by include_dir! in server/src/http_server.rs
    emptyOutDir: true,
    rollupOptions: {
      output: {
        entryFileNames: "app.js", // Set the output file name to app.js
        assetFileNames: "[name][extname]", // Prevent assets from going into 'assets' folder
      },
    },
  },
});
