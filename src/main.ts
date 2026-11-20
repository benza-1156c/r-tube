import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
import "./style.css";
import { useSettings } from "./composables/useSettings.ts";

async function bootstrap() {
  const { loadSettings } = useSettings();

  await loadSettings();

  const app = createApp(App);
  app.use(router);
  app.mount("#app");
}

bootstrap();
