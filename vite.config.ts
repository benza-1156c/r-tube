import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import process from "node:process";
import tailwindcss from "@tailwindcss/vite";
import path from "node:path";
import VueRouter from "unplugin-vue-router/vite";
import { VueRouterAutoImports } from "unplugin-vue-router";
import AutoImport from "unplugin-auto-import/vite";
import Components from "unplugin-vue-components/vite";

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(() => ({
  plugins: [
    VueRouter({
      routesFolder: "src/router",
      extensions: [".vue"],
      dts: "src/types/typed-router.d.ts",
      extendRoute(route) {
        if (route.path.startsWith("/watch")) {
          route.addToMeta({ hideSidebar: true });
        }
      },
    }),
    vue({
      template: {
        compilerOptions: {
          isCustomElement: (tag) =>
            tag.startsWith("video-") ||
            tag.startsWith("media-") ||
            tag === "hlsjs-video" ||
            tag === "dash-video",
        },
      },
    }),
    tailwindcss(),
    AutoImport({
      imports: [
        "vue",
        VueRouterAutoImports,
        "@vueuse/core",
      ],
      dirs: [
        "src/lib",
      ],
      dts: "src/types/auto-imports.d.ts",
      vueTemplate: true,
    }),
    Components({
      dirs: [
        "src/components",
        "src/lib/components",
      ],
      extensions: ["vue"],
      deep: true,
      dts: "src/types/components.d.ts",
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(import.meta.dirname, "./src"),
    },
  },
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
