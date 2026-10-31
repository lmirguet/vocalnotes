import { fileURLToPath } from "node:url";
import { defineConfig } from "vite";

const local = (path: string) => fileURLToPath(new URL(path, import.meta.url));

export default defineConfig({
  // Never load project .env files, copy public/, or serve the project root.
  envDir: local("./tooling/empty-env"),
  envPrefix: [],
  publicDir: false,
  clearScreen: false,
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
    fs: {
      strict: true,
      allow: [local("./src"), local("./node_modules"), local("./index.html")],
      deny: ["**/.env*", "**/*key*", "**/*.key", "**/.git/**", "**/.specify/**", "**/specs/**"],
    },
  },
  preview: { host: "127.0.0.1", port: 1421, strictPort: true },
  build: {
    target: "es2022",
    outDir: "dist",
    emptyOutDir: true,
    sourcemap: false,
  },
});
