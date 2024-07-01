<template>
  <nav>
    <ul class="tabs flex flex-row cursor-pointer">
      <li @contextmenu="contextMenu($event, Folder.Clipboard)" @click="switchTab(Folder.Clipboard)"
        class="relative p-3 px-5 text-xs sm:text-base border-b flex shrink-0"
        :class="{ active: activeTabId === Folder.Clipboard, 'border-gray-500': activeTabId !== Folder.Clipboard }">
        <div class="z-1">Clipboard</div>
        <div v-if="data[Folder.Clipboard].children.length"
          class="opacity-80 z-0 chip absolute text-[10px] m-0 p-0 right-0 top-1 px-[2px] rounded-md">
          {{ data[Folder.Clipboard].children.length }}
        </div>
      </li>
      <li @contextmenu="contextMenu($event, Folder.Clipboard)" @click="switchTab(Folder.Favorites)"
        class="relative p-3 px-5 text-xs sm:text-base border-b flex shrink-0"
        :class="{ active: activeTabId === Folder.Favorites, 'border-gray-500': activeTabId !== Folder.Favorites }">
        <img src="./assets/star.svg" alt="" class="w-4 mt-[-3px] mr-1">
        <div class="z-1">Favorites</div>
        <div v-if="data[Folder.Favorites].children.length"
          class="opacity-80 z-0 chip absolute text-[10px] m-0 p-0 right-0 top-1 px-[2px] rounded-md">
          {{ data[Folder.Favorites].children.length }}
        </div>
      </li>
      <!-- <li class="p-3 px-5 text-xs sm:text-base border-b border-gray-500 flex shrink-0">
        <img src="./assets/add.svg" alt="" class="mt-[-3px]">
        Add
      </li> -->
      <li class="w-[-webkit-fill-available] p-3 px-5 border-b border-gray-500"></li>
      <li @click="menuType = MENU_TYPE.Main" class="py-2 border-b border-gray-500">
        <svg xmlns="http://www.w3.org/2000/svg" width="2rem" height="20px" preserveAspectRatio="xMidYMid meet">
          <path fill="white" d="M3 18v-2h18v2Zm0-5v-2h18v2Zm0-5V6h18v2Z" />
        </svg>
      </li>
    </ul>

    <div class="search flex flex-row p-2">
      <input type="text" placeholder="Search" class="p-1 text-xs sm:text-base w-11/12 border-sky-500">
      <img src="./assets/search.svg" alt=""
        class="w-8 max-h-10 sm:w-1/12 pl-2 opacity-30 hover:opacity-100 cursor-pointer">
    </div>
  </nav>

  <main class="ml-2 mr-1 overflow-y-scroll overflow-x-hidden pr-1">
    <ul v-if="data[activeTabId] && data[activeTabId].children">
      <li v-for="(item, key) in data[activeTabId].children" :key="key" class="flex pb-2 mb-2 border-b border-neutral-700">
        <div class="item w-11/12 cursor-pointer" @click="pasteItem(item)">
          <div class="value text-xs sm:text-base pb-2 mb-2 leading-5 overflow-hidden max-h-14">{{ item.contents }}</div>
          <div class="meta text-xs text-neutral-500">{{ formatDate(getTimestamp(item.name)) }}</div>
        </div>
        <div class="controls flex items-center">
          <button @click="bookmarkItem(item)"><img src="./assets/star.svg" alt="Bookmark"></button>
          <button @click="deleteItem(item)"><img src="./assets/trash.svg" alt="Delete"></button>
        </div>
      </li>
    </ul>
  </main>

  <app-popup :type="menuType" :currentFolder="contextMenuFolder" @click="menuType = MENU_TYPE.None" />
</template>

<script setup lang="ts">
import { register, unregister } from '@tauri-apps/api/globalShortcut';
import { listen } from '@tauri-apps/api/event'
import { readDir, BaseDirectory, FileEntry, readTextFile } from '@tauri-apps/api/fs';
import { appWindow } from '@tauri-apps/api/window'
import { computed, nextTick } from '@vue/runtime-core';
import { ref } from 'vue';
import { DIR_DATA, Folder, FOLDER_NAME, FOLDER_NAME_MAP, MENU_TYPE } from './common/constants';
import { ClipboardData, ClipboardFolder } from './common/interfaces';
import AppPopup from './components/AppPopup.vue';

const invoke = window.__TAURI__.invoke;

const menuType = ref(0);

const contextMenuFolder = ref(0);

const activeTabId = ref(Folder.Clipboard);

const data = ref<null | ClipboardData | FileEntry[]>(null);

const contextMenu = (e: PointerEvent, id: number) => {
  contextMenuFolder.value = id || 0;
  menuType.value = MENU_TYPE.Context;
}

const formatDate = (timestamp: string) => {
  var date = new Date(parseInt(timestamp));
  var hours = date.getHours();
  var minutes = "0" + date.getMinutes();
  var seconds = "0" + date.getSeconds();

  var y = date.getFullYear();
  var m = "0" + (date.getMonth() + 1);
  var d = "0" + date.getDate();

  return `${y}-${m.substr(-2)}-${d.substr(-2)} ` + hours + ':' + minutes.substr(-2) + ':' + seconds.substr(-2);

}

const getTimestamp = (filename: string) => filename.split('.').slice(0, -1).join('.');

const isImage = (filename: string) => !filename.includes(".txt");

const fetchData = () => {
  return new Promise(async (resolve, reject) => {
    try {
      data.value = await readDir(DIR_DATA, { dir: BaseDirectory.AppLocalData, recursive: true });
      console.log("FETCH", data.value);

      if (!data.value || !data.value[Folder.Clipboard] || !data.value[Folder.Favorites]) {
        resolve([]);
      }

      for (const [f, folder] of Object.entries(data.value)) {
        for (const [c, file] of Object.entries((folder as ClipboardFolder).children)) {
          const contents = await readTextFile(file.path);
          data.value[f].children[c].contents = contents;
          data.value[f].children[c].folder = folder.name;
        }
      }

      resolve(data.value);
    } catch (e) {
      reject(new Error(e));
    }

  });
}

const switchTab = async (tabId: number) => {
  if (tabId !== activeTabId.value) {
    await fetchData();
    activeTabId.value = tabId;
  }
}

const pasteItem = async (item: ClipboardItem) => {
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
  console.log(`REMOVE: ${item.folder}/${item.name}`);

  invoke("remove_clipboard_item", {
    filename: item.name,
    folder: item.folder,
  });
}

(async () => {
  await listen('clipboard', async (event: any) => {
    const unlisten = console.log("EVENT", event.message);
    await fetchData();
  })

  const unlisten2 = await listen('clipboard_img', (event: any) => {
    console.log("EVENT!", [...event.message]);
  })

  await register('CommandOrControl+1', () => {
    console.log('Shortcut triggered');
    invoke("show_window");
  });

  invoke('enable_clipboard');

  await fetchData();
})()

// document.addEventListener('contextmenu', event => event.preventDefault());

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

.active {
  @apply text-sky-500 border-sky-500;
}
</style>
