import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
import "./style.css";
import { useSettings } from "./composables/useSettings.ts";
import { applyTheme, getTheme } from "./lib/theme.ts";

// Apply the saved theme before the first paint so opening the app never
// flashes the other theme.
applyTheme(getTheme());

async function bootstrap() {
  const { loadSettings } = useSettings();

  await loadSettings();

  const app = createApp(App);
  app.use(router);
  app.mount("#app");
}

bootstrap();
