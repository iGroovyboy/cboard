<template>
  <app-titlebar />

  <app-tabs :active-tab-id="activeTabId" :clip-len="data?.[Folder.Clipboard]?.children?.length"
    :fav-len="data?.[Folder.Favorites]?.children?.length" @switch-tab="switchTab" @mainmenu="menuType = MENU_TYPE.Main"
    @contextmenu="contextMenu" />

  <div class="search flex flex-row p-2">
    <input type="text" placeholder="Search" class="p-1 text-xs sm:text-base w-11/12 border-sky-500">
    <img src="./assets/search.svg" alt="" class="w-8 max-h-10 sm:w-1/12 pl-2 opacity-30 hover:opacity-100 cursor-pointer">
  </div>

  <main class="ml-2 mr-1 overflow-y-scroll overflow-x-hidden pr-1">
    <ul v-if="data && data[activeTabId] && data[activeTabId].children">
      <li v-for="( item, key ) in  data[activeTabId].children " :key="key"
        class="flex pl-1 pb-2 mb-2 border border-transparent border-b border-b-neutral-700"
        :class="{ 'border border-white/50 border-b-white/50': key === focusedElementId }">
        <div class="item w-11/12 cursor-pointer" @click="pasteItem(item)">
          <div class="value text-xs sm:text-base pb-2 mb-2 leading-5 overflow-hidden"
            :class="{ 'max-h-14': item.type === FILE_EXT.TXT }">
            <template v-if="item.type === FILE_EXT.TXT">{{ item.contents }}</template>
            <template v-else-if="item.type === FILE_EXT.PNG">
              <img :src="item.contents" class="border border-white/50 hover:border-white" alt="image">
            </template>
          </div>
          <div class="meta text-xs text-neutral-500">{{ formatDate(getTimestamp(item.name)) }}</div>
        </div>
        <div class="controls flex items-center">
          <button @click="bookmarkItem(item)"><img src="./assets/star.svg" alt="Bookmark"></button>
          <button @click="deleteItem(item)"><img src="./assets/trash.svg" alt="Delete"></button>
        </div>
      </li>
    </ul>
  </main>

  <app-popup ref="popup" :type="menuType" :currentFolder="contextMenuFolder" @click="menuType = MENU_TYPE.None"
    @close="menuType = MENU_TYPE.None" />
</template>

<script setup lang="ts">
import { register, unregister } from '@tauri-apps/api/globalShortcut';
import { listen } from '@tauri-apps/api/event'
import { FileEntry } from '@tauri-apps/api/fs';
import { appWindow, LogicalPosition, LogicalSize } from '@tauri-apps/api/window'
import { onBeforeMount, ref } from 'vue';
import { FILE_EXT, Folder, FOLDER_NAME, MENU_TYPE } from './common/constants';
import { ClipboardData, SETTINGS_KEY } from './common/interfaces';
import { getFilesData } from './services/backend';
import { debounce, formatDate } from './common/helpers';
import AppTitlebar from './components/AppTitlebar.vue';
import AppPopup from './components/AppPopup.vue';
import AppTabs from './components/AppTabs.vue';
import ls from './common/localStorage';

const invoke = window.__TAURI__.invoke;

const menuType = ref(0);

const contextMenuFolder = ref(0);

const activeTabId = ref<number>(Folder.Clipboard);

const data = ref<null | ClipboardData | FileEntry[]>(null);

const focusedElementId = ref<null | number>(null);

let debounceTimeout: number;

let windowPos = {};

const contextMenu = (e: PointerEvent, id: number) => {
  contextMenuFolder.value = id || 0;
  menuType.value = MENU_TYPE.Context;
}

const getTimestamp = (filename: string) => filename.split('.').slice(0, -1).join('.');

const isImage = (filename: string) => !filename.includes(".txt");

const fetchData = async () => {
  data.value = await getFilesData();
};

const switchTab = async (tabId: number) => {
  if (tabId !== activeTabId.value) {
    await fetchData();
    activeTabId.value = tabId;
  }
}

const toggleNextTab = () => {
  activeTabId.value = +!!!activeTabId.value;
}

const pasteItem = async (item: ClipboardItem) => {
  if (!item) {
    return;
  }

  await appWindow.hide();
  invoke("paste", { item: item });
}

const moveItemToFolder = (item: ClipboardItem) => {
  invoke("move_clipboard_item", {
    from: item.path,
    filename: item.name,
    folder: FOLDER_NAME.Favorites,
  });
}

const deleteItem = (item: ClipboardItem) => {
  if (!item) {
    return;
  }
  console.log(`REMOVE: ${item.folder}/${item.name}`);

  invoke("remove_clipboard_item", {
    filename: item.name,
    folder: item.folder,
  });
}

const keysBoot = () => {
  document.addEventListener('keydown', event => {
    switch (event.key) {
      case "ArrowDown":
        if (data.value[activeTabId.value]?.children?.length - 1 === focusedElementId.value) {
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
          pasteItem(data.value[activeTabId.value]?.children?.[+focusedElementId.value]);
        }
        break;
      case "Delete":
        if (focusedElementId.value !== null) {
          deleteItem(data.value[activeTabId.value]?.children?.[+focusedElementId.value]);
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
}


const bootUp = async () => {
  document.addEventListener('contextmenu', event => event.preventDefault());

  await listen('clipboard', async (event: any) => {
    const unlisten = console.log("EVENT", event);
    await fetchData();
  })

  await listen('clipboard_img', (event: any) => {
    console.log("EVENT!", [...event.message]);
  })

  appWindow.onMoved(debounce(({ payload: position }) => {
    console.log('Window moved to:', position);
    ls.save(SETTINGS_KEY.WINDOW_POS, position);
  }, 500));

  appWindow.onResized(debounce(({ payload: size }) => {
    console.log('Window resized:', size);
    ls.save(SETTINGS_KEY.WINDOW_SIZE, size);
  }, 500));

  if (ls.has(SETTINGS_KEY.WINDOW_POS)) {
    const pos = ls.get(SETTINGS_KEY.WINDOW_POS);
    await appWindow.setPosition(new LogicalPosition(pos.x, pos.y));
  }

  if (ls.has(SETTINGS_KEY.WINDOW_SIZE)) {
    const size = ls.get(SETTINGS_KEY.WINDOW_SIZE);
    await appWindow.setSize(new LogicalSize(size.width, size.height));
  }

  keysBoot();

  // TODO: make customizable
  await register('CommandOrControl+1', () => {
    console.log('Shortcut triggered');
    invoke("show_window");
  });

  await appWindow.setAlwaysOnTop(true);

  invoke('enable_clipboard');

  await fetchData();
}

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
