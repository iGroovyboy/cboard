<template>
  <nav>
    <ul class="tabs flex flex-row cursor-pointer">
      <li class="p-3 px-5 text-xs border-b border-sky-500 text-sky-500">Clipboard</li>
      <li class="p-3 px-5 text-xs border-b border-gray-500 flex">
        <img src="./assets/star.svg" alt="" class="w-4 mt-[-3px] mr-1">
        Favorites
      </li>
      <li class="p-3 px-5 text-xs border-b border-gray-500 flex">
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
    <ul v-if="data[0] && data[0].children">
      <li v-for="(item, key) in data[0].children" :key="key" class="flex pb-2 mb-2 border-b border-neutral-700">
        <div class="item w-11/12 cursor-pointer">
          <div class="value text-xs pb-2 mb-2 leading-5 overflow-hidden max-h-14">{{ item.contents }}</div>
          <div class="meta text-xs text-neutral-500">{{ formatDate(getTimestamp(item.name)) }}</div>
        </div>
        <div class="controls flex items-center">
          <button><img src="./assets/star.svg" alt="Bookmark"></button>
          <button><img src="./assets/trash.svg" alt="Delete"></button>
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

const fetchData = async () => { 
  data.value = await readDir('data', { dir: BaseDirectory.AppLocalData, recursive: true });
  console.log("FETCH", data.value);
  
  if (!data.value || !data.value[0]) {
    return [];
  }

  for (const [key, file] of Object.entries(data.value[0].children)) {
    const contents = await readTextFile(file.path);
    data.value[0].children[key].contents = contents;
  }

 }

(async() => { 
  const invoke = window.__TAURI__.invoke

  await listen('clipboard', (event: any) => {
     const unlisten = console.log(">>> EVENT", event.message);
     fetchData();
  })
  
  const unlisten2 = await listen('clipboard_img', (event: any) => {
    console.log(">>> EVENT!", [...event.message]);   
  })

  invoke('enable_clipboard');

  // Reads the `$APPDATA/users` directory recursively
  // const entries = await readDir('data', { dir: BaseDirectory.AppLocalData, recursive: true });
  // console.log('>>>', entries);
  

  // function processEntries(entries: any) {
  //   for (const entry of entries) {
  //     console.log(`Entry: ${entry.path}`);
  //     if (entry.children) {
  //       processEntries(entry.children)
  //     }
  //   }
  // }

  // processEntries(entries);
  fetchData();
})()


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

</style>
