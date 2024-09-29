import { createRouter, createWebHistory } from "vue-router";
import Clipboard from "../components/AppClipboard.vue";
import Autoreplacement from "../components/AppAutoreplacement.vue";
import Settings from "../components/AppSettings.vue";
import Blacklist from "../components/AppBlacklist.vue";
import KeyboardLayouts from "../components/AppKeyboardLayouts.vue";
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
  {
    path: "/blacklist",
    name: ROUTE.Blacklist,
    component: Blacklist,
  },
  {
    path: "/keyboard_layouts",
    name: ROUTE.KeyboardLayouts,
    component: KeyboardLayouts,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
