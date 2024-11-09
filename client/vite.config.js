// vite.config.js
import { defineConfig } from "vite";

export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        entryFileNames: "app.js", // Set the output file name to app.js
        assetFileNames: "[name][extname]", // Prevent assets from going into 'assets' folder
      },
    },
  },
});
