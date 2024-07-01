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
            <li @click="mainMenuShown = !mainMenuShown" class="py-2 border-b border-gray-500">
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
</template>

<script setup lang="ts">
</script>