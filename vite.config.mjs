import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;
const devHost = host || "127.0.0.1";

export default defineConfig({
  root: "src",
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: devHost,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: "src/index.html",
        cps: "src/cps.html",
      },
    },
  },
});
