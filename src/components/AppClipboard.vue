<template>
  <app-tabs
    :active-tab-id="activeTabId"
    :clip-len="data?.[Folder.Clipboard]?.children?.length"
    :fav-len="data?.[Folder.Favorites]?.children?.length"
    @switch-tab="switchTab"
    @mainmenu="menuType = MENU_TYPE.Main"
    @contextmenu="contextMenu"
  />

  <div class="search flex flex-row p-2">
    <input
      type="text"
      placeholder="Search"
      class="p-1 text-xs sm:text-base w-11/12 border-sky-500"
    />
    <img
      src="./assets/search.svg"
      alt=""
      class="w-8 max-h-10 sm:w-1/12 pl-2 opacity-30 hover:opacity-100 cursor-pointer"
    />
  </div>

  <main class="ml-2 mr-1 overflow-y-scroll overflow-x-hidden pr-1">
    <ul v-if="data && data[activeTabId] && data[activeTabId].children">
      <li
        v-for="(item, key) in data[activeTabId].children"
        :key="key"
        class="flex pl-1 pb-2 mb-2 border border-transparent border-b border-b-neutral-700"
        :class="{
          'border border-white/50 border-b-white/50': key === focusedElementId,
        }"
      >
        <div class="item w-11/12 cursor-pointer" @click="pasteItem(item)">
          <div
            class="value text-xs sm:text-base pb-2 mb-2 leading-5 overflow-hidden"
            :class="{ 'max-h-14': item.type === FILE_EXT.TXT }"
          >
            <template v-if="item.type === FILE_EXT.TXT">{{
              item.contents
            }}</template>
            <template v-else-if="item.type === FILE_EXT.PNG">
              <img
                :src="item.contents"
                class="border border-white/50 hover:border-white"
                alt="image"
              />
            </template>
          </div>
          <div class="meta text-xs text-neutral-500">
            {{ formatDate(getTimestamp(item.name)) }}
          </div>
        </div>
        <div class="controls flex items-center">
          <button
            class="p-1 w-6 ml-1 opacity-50 hover:opacity-100"
            @click="bookmarkItem(item)"
          >
            <img src="../assets/star.svg" alt="Bookmark" />
          </button>
          <button
            class="p-1 w-6 ml-1 opacity-50 hover:opacity-100"
            @click="deleteItem(item)"
          >
            <img src="../assets/trash.svg" alt="Delete" />
          </button>
        </div>
      </li>
    </ul>
  </main>

  <app-popup
    ref="popup"
    :type="menuType"
    :currentFolder="contextMenuFolder"
    @click="menuType = MENU_TYPE.None"
    @close="menuType = MENU_TYPE.None"
  />
</template>

<script setup lang="ts">
import AppTabs from "./AppTabs.vue";
import { ref } from "vue";
import { FILE_EXT, Folder, FOLDER_NAME, MENU_TYPE } from "../common/constants";
import { ClipboardData } from "../common/interfaces";
import { FileEntry } from "@tauri-apps/api/fs";
import { getFilesData } from "../services/backend";
import { appWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { formatDate } from "../common/helpers";
import AppPopup from "./AppPopup.vue";

const invoke = window.__TAURI__.invoke;

const activeTabId = ref<number>(Folder.Clipboard);

const menuType = ref(0);

const contextMenuFolder = ref(0);

const data = ref<null | ClipboardData | FileEntry[]>(null);

const focusedElementId = ref<null | number>(null);

let debounceTimeout: number;

const contextMenu = (e: PointerEvent, id: number) => {
  contextMenuFolder.value = id || 0;
  menuType.value = MENU_TYPE.Context;
};

const getTimestamp = (filename: string) =>
  filename.split(".").slice(0, -1).join(".");

const fetchData = async () => {
  data.value = await getFilesData();
};

const switchTab = async (tabId: number) => {
  if (tabId !== activeTabId.value) {
    await fetchData();
    activeTabId.value = tabId;
  }
};

const toggleNextTab = () => {
  activeTabId.value = +!!!activeTabId.value;
};

const bookmarkItem = () => {
  //...
};

const pasteItem = async (item: ClipboardItem) => {
  if (!item) {
    return;
  }

  await appWindow.hide();
  invoke("paste", { item: item });
};

const moveItemToFolder = (item: ClipboardItem) => {
  invoke("move_clipboard_item", {
    from: item.path,
    filename: item.name,
    folder: FOLDER_NAME.Favorites,
  });
};

const deleteItem = (item: ClipboardItem) => {
  if (!item) {
    return;
  }
  console.log(`REMOVE: ${item.folder}/${item.name}`);

  invoke("remove_clipboard_item", {
    filename: item.name,
    folder: item.folder,
  });
};

const keysBoot = () => {
  document.addEventListener("keydown", (event) => {
    switch (event.key) {
      case "ArrowDown":
        if (
          data.value[activeTabId.value]?.children?.length - 1 ===
          focusedElementId.value
        ) {
          break;
        }
        focusedElementId.value += 1;
        console.log(focusedElementId.value);
        break;
      case "ArrowUp":
        if (focusedElementId.value === 0) {
          break;
        }
        focusedElementId.value -= 1;
        console.log(focusedElementId.value);
        break;
      case " " || "Enter":
        if (focusedElementId.value !== null) {
          pasteItem(
            data.value[activeTabId.value]?.children?.[+focusedElementId.value],
          );
        }
        break;
      case "Delete":
        if (focusedElementId.value !== null) {
          deleteItem(
            data.value[activeTabId.value]?.children?.[+focusedElementId.value],
          );
        }
        break;
      case "Tab":
        event.preventDefault();
        toggleNextTab();
        break;
      default:
        break;
    }
  });
};

const bootUp = async () => {
  await listen("clipboard", async (event: any) => {
    const unlisten = console.log("EVENT", event);
    await fetchData();
  });

  await listen("clipboard_img", (event: any) => {
    console.log("EVENT!", [...event.message]);
  });

  keysBoot();

  await fetchData();
};

bootUp();
</script>

<style scoped lang="scss"></style>
