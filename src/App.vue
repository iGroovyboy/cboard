<template>
  <app-titlebar />

  <div class="wrapper">
    <router-view></router-view>
  </div>
</template>

<script setup lang="ts">
import { register, unregister } from "@tauri-apps/api/globalShortcut";
import {
  appWindow,
  LogicalPosition,
  LogicalSize,
} from "@tauri-apps/api/window";
import { ref } from "vue";
import { MENU_TYPE } from "./common/constants";
import { SETTINGS_KEY } from "./common/interfaces";
import { debounce } from "./common/helpers";
import AppTitlebar from "./components/AppTitlebar.vue";

import ls from "./common/localStorage";

const invoke = window.__TAURI__.invoke;

let windowPos = {};

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

  if (ls.has(SETTINGS_KEY.WINDOW_POS)) {
    const pos = ls.get(SETTINGS_KEY.WINDOW_POS);
    await appWindow.setPosition(new LogicalPosition(pos.x, pos.y));
  }

  if (ls.has(SETTINGS_KEY.WINDOW_SIZE)) {
    const size = ls.get(SETTINGS_KEY.WINDOW_SIZE);
    await appWindow.setSize(new LogicalSize(size.width, size.height));
  }

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

button {
  padding: 3px;
  width: 25px;
  height: 25px;
  margin-left: 5px;
  opacity: 0.5;

  &:hover {
    opacity: 1;
  }
}
</style>
