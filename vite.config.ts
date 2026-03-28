import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import Components from "unplugin-vue-components/vite";
import { AntDesignVueResolver } from "unplugin-vue-components/resolvers";
import UnoCSS from "unocss/vite";
import { resolve } from "path";

// process 是 Node.js 的全局对象
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    Components({
      //  自动导入组件
      resolvers: [
        AntDesignVueResolver({
          importStyle: false, // 在 JS 中引入 CSS
        }),
      ],
    }),
    UnoCSS(),
  ],

  // 为 Tauri 开发定制的 Vite 配置，仅在 `tauri dev` 或 `tauri build` 时生效
  //
  // 1. 防止 Vite 清屏后遮挡 Rust 报错信息
  clearScreen: false,
  // 2. Tauri 依赖固定端口，如果端口不可用则直接报错
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
      // 3. 告诉 Vite 忽略对 `src-tauri` 的监听
      ignored: ["**/src-tauri/**"],
    },
  },
  // 配置@规则
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
}));
