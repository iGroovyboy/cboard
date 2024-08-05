<template>
  <app-titlebar />

  <div class="wrapper flex flex-col h-[calc(100vh-32px)]">
    <router-view :key="$route.fullPath" />
  </div>
</template>

<script setup lang="ts">
import { register } from "@tauri-apps/api/globalShortcut";
import {
  appWindow,
  LogicalPosition,
  LogicalSize,
} from "@tauri-apps/api/window";
import { SETTINGS_KEY } from "./common/interfaces";
import { debounce } from "./common/helpers";
import AppTitlebar from "./components/AppTitlebar.vue";

import ls from "./common/localStorage";
import { onMounted } from "vue";

const invoke = window.__TAURI__.invoke;

const bootUp = async () => {
  document.addEventListener("contextmenu", (event) => event.preventDefault());

  appWindow.onMoved(
    debounce(({ payload: position }) => {
      console.log("Window moved to:", position);
      ls.save(SETTINGS_KEY.WINDOW_POS, position);
    }, 500),
  );

  appWindow.onResized(
    debounce(({ payload: size }) => {
      console.log("Window resized:", size);
      ls.save(SETTINGS_KEY.WINDOW_SIZE, size);
    }, 500),
  );

  // TODO: fix: uncommented this causes innere components to not update until window title clicked or updated
  // if (ls.has(SETTINGS_KEY.WINDOW_POS)) {
  //   const pos = ls.get(SETTINGS_KEY.WINDOW_POS);
  //   await appWindow.setPosition(new LogicalPosition(pos.x, pos.y));
  // }
  // if (ls.has(SETTINGS_KEY.WINDOW_SIZE)) {
  //   const size = ls.get(SETTINGS_KEY.WINDOW_SIZE);
  //   await appWindow.setSize(new LogicalSize(size.width, size.height));
  // }

  // TODO: make customizable
  await register("CommandOrControl+1", () => {
    console.log("Shortcut triggered");
    invoke("show_window");
  });

  await appWindow.setAlwaysOnTop(true);

  invoke("enable_clipboard");
};

bootUp();
</script>

<style lang="scss" scoped>
main {
  height: calc(100vh - 90px);
}
</style>
