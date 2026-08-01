import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import Icons from "unplugin-icons/vite";
import { defineConfig } from "vite";

// https://vite.dev/config/
export default defineConfig({
  envDir: "../",
  plugins: [tailwindcss(), svelte(), Icons({ compiler: "svelte" })],
});
