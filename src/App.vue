<template>
  <app-titlebar />

  <app-tabs :active-tab-id="activeTabId" :clip-len="data?.[Folder.Clipboard]?.children?.length"
    :fav-len="data?.[Folder.Favorites]?.children?.length" @switch-tab="switchTab" @mainmenu="menuType = MENU_TYPE.Main"
    @contextmenu="contextMenu" />

  <main class="ml-2 mr-1 overflow-y-scroll overflow-x-hidden pr-1">
    <ul v-if="data && data[activeTabId] && data[activeTabId].children">
      <li v-for="( item, key ) in  data[activeTabId].children " :key="key"
        class="flex pb-2 mb-2 border-b border-neutral-700">
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
import { appWindow } from '@tauri-apps/api/window'
import { onBeforeMount, ref } from 'vue';
import { FILE_EXT, Folder, FOLDER_NAME, MENU_TYPE } from './common/constants';
import { ClipboardData } from './common/interfaces';
import { getFilesData } from './services/backend';
import { formatDate } from './common/helpers';
import AppTitlebar from './components/AppTitlebar.vue';
import AppPopup from './components/AppPopup.vue';
import AppTabs from './components/AppTabs.vue';

const invoke = window.__TAURI__.invoke;

const menuType = ref(0);

const contextMenuFolder = ref(0);

const activeTabId = ref(Folder.Clipboard);

const data = ref<null | ClipboardData | FileEntry[]>(null);

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

const bootUp = async () => {
  document.addEventListener('contextmenu', event => event.preventDefault());

  await listen('clipboard', async (event: any) => {
    const unlisten = console.log("EVENT", event);
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
