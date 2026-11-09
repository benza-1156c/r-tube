import "vue-router";

/**
 * RouteMeta declaration for Vue Router.
 * Configures layout display properties, including whether
 * a specific page hides or shows the sidebar.
 */
declare module "vue-router" {
  interface RouteMeta {
    /**
     * Page title for header and browser document.
     */
    title?: string;

    /**
     * Whether this route completely hides the sidebar.
     * - `true`: Hides the sidebar (e.g. watch page `/watch`).
     * - `false` or `undefined`: Displays the sidebar normally.
     */
    hideSidebar?: boolean;

    /**
     * Automatically collapse the sidebar into icon-only mode.
     */
    collapseSidebar?: boolean;

    /**
     * Navigation menu grouping (e.g. 'main', 'library', 'settings').
     */
    group?: "main" | "library" | "settings";

    /**
     * Whether the page requires authentication before access.
     */
    requiresAuth?: boolean;
  }
}

export interface NavItemConfig {
  title: string;
  path: string;
  icon: any;
  badge?: string;
  group?: "main" | "library" | "settings";
}
