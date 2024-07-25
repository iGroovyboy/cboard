import { createRouter, createWebHistory } from "vue-router";
import Clipboard from "../components/AppClipboard.vue";
import Autoreplacement from "../components/AppAutoreplacement.vue";
import Settings from "../components/AppSettings.vue";
import { ROUTE } from "./routenames";

const routes = [
  {
    path: "/",
    name: ROUTE.Home,
    component: Clipboard,
  },
  {
    path: "/autoreplace",
    name: ROUTE.Autoreplace,
    component: Autoreplacement,
  },
  {
    path: "/settings",
    name: ROUTE.Settings,
    component: Settings,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
