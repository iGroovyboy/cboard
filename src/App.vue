<template>
  <nav>
    <ul class="tabs flex flex-row cursor-pointer">
      <li @click="switchTab(Folder.Clipboard)" 
        class="p-3 px-5 text-xs border-b flex shrink-0" 
        :class="{active: activeTabId === Folder.Clipboard, 'border-gray-500': activeTabId !== Folder.Clipboard}">
        Clipboard
      </li>
      <li @click="switchTab(Folder.Favorites)" 
        class="p-3 px-5 text-xs border-b flex shrink-0"
        :class="{active: activeTabId === Folder.Favorites, 'border-gray-500': activeTabId !== Folder.Favorites}">
        <img src="./assets/star.svg" alt="" class="w-4 mt-[-3px] mr-1">
        Favorites
      </li>
      <li class="p-3 px-5 text-xs border-b border-gray-500 flex shrink-0">
        <img src="./assets/add.svg" alt="" class="mt-[-3px]">
        Add
      </li>
      <li class="w-[-webkit-fill-available] p-3 px-5 border-b border-gray-500"></li>
    </ul>

    <div class="search flex flex-row p-2">
      <input type="text" placeholder="Search" class="p-1 text-xs w-11/12 border-sky-500">
      <img src="./assets/search.svg" alt="" class="w-1/12 h-25 pl-2 opacity-30 hover:opacity-100">
    </div>
  </nav>

  <main class="mx-2 overflow-y-scroll h-80 pr-1">
    <ul v-if="data[activeTabId] && data[activeTabId].children">
      <li v-for="(item, key) in data[activeTabId].children" :key="key" class="flex pb-2 mb-2 border-b border-neutral-700">
        <div class="item w-11/12 cursor-pointer" @click="pasteItem(item)">
          <div class="value text-xs pb-2 mb-2 leading-5 overflow-hidden max-h-14">{{ item.contents }}</div>
          <div class="meta text-xs text-neutral-500">{{ formatDate(getTimestamp(item.name)) }}</div>
        </div>
        <div class="controls flex items-center">
          <button @click="bookmarkItem(item)"><img src="./assets/star.svg" alt="Bookmark"></button>
          <button @click="deleteItem(item)"><img src="./assets/trash.svg" alt="Delete"></button>
        </div>
      </li>
    </ul>
  </main>
</template>

<script setup lang="ts">
import { register, unregister } from '@tauri-apps/api/globalShortcut';
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'
import { readDir, BaseDirectory, FileEntry, readTextFile } from '@tauri-apps/api/fs';
import { computed } from '@vue/runtime-core';
import { ref } from 'vue';

const invoke = window.__TAURI__.invoke;

const activeTabId = ref(Folder.Clipboard);

const data = ref<null | FileEntry>(null);

const formatDate = (timestamp: any) => { 
  var date = new Date(parseInt(timestamp));
    var hours = date.getHours();
    var minutes = "0" + date.getMinutes();
    var seconds = "0" + date.getSeconds();
    
    var y = date.getFullYear();
    var m = "0" +( date.getMonth() + 1);
    var d = "0" + date.getDate();

    return `${y}-${m.substr(-2)}-${d.substr(-2)} ` + hours + ':' + minutes.substr(-2) + ':' + seconds.substr(-2);

}

const getTimestamp = (filename: string) => filename.split('.').slice(0, -1).join('.');

const isImage = (filename: string) => !filename.includes(".txt");

enum Folder {
  Clipboard = 0,
  Favorites = 1,
}

const FOLDER_NAME = {
  Clipboard: "clipboard",
  Favorites: "favorites",
}

const fetchData = () => { 
  return new Promise(async (resolve, reject) => {
    try {
      data.value = await readDir('data', { dir: BaseDirectory.AppLocalData, recursive: true });
      console.log("FETCH", data.value);
      
      if (!data.value || !data.value[Folder.Clipboard] || !data.value[Folder.Favorites]) {
        resolve([]);
      }

      for (const [f, folder] of Object.entries(data.value)) {
        for (const [c, file] of Object.entries(folder.children)) {
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

(async() => { 

  await listen('clipboard', async (event: any) => {
     const unlisten = console.log("EVENT", event.message);
     await fetchData();
  })
  
  const unlisten2 = await listen('clipboard_img', (event: any) => {
    console.log("EVENT!", [...event.message]);   
  })

  invoke('enable_clipboard');

  await fetchData();
})()

const pasteItem = (item: any) => {  }

const moveItemToFolder = (item: any) => { 
  invoke("move_clipboard_item", { 
    from: item.path,
    filename: item.name,
    folder: FOLDER_NAME.Favorites,
  });
}

const deleteItem = (item: any) => {
  console.log(`REMOVE: ${item.folder}/${item.name}`);
  
  invoke("remove_clipboard_item", { 
    filename: item.name,
    folder: item.folder,
  });
}

</script>

<style lang="scss" scoped>
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
