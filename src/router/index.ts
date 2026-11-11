import { ref } from "vue";
import { createRouter, createWebHistory } from "vue-router";
import { routes } from "vue-router/auto-routes";

export const router = createRouter({
  history: createWebHistory(),
  routes,
  scrollBehavior() {
    return { top: 0 };
  },
});

export { routes };

export const previousPath = ref("");

router.beforeEach((_to, from) => {
  previousPath.value = from.path;
  return true;
});
