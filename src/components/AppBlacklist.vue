<template>
    <app-headerbar title="Blacklist applications" />

    <div v-if="!isProcessesVisible" class="blacklisted-apps flex flex-col overflow-y-scroll flex-grow pb-16">
        <div class="apps pt-0.5 overflow-x-hidden flex flex-col flex-grow">
            <div v-for="(item, i) in blacklistedApps" :key="i" @mousedown="selectedItemId = i"
                class="item flex gap-x-2 px-1 mb-1 items-center leading-5"
                :class="{ 'bg-sky-600 text-white': selectedItemId == i }">
                <input class="min-h-4 min-w-4" type="checkbox" v-model="item.enabled" @change="toggleApp(item)">
                <span v-text="item.title || item.filename"
                    :title="(item.title || item.filename) + `   (${item.filename})`" class="text-sm text-nowrap"
                    :class="{ 'text-white/40': !item.enabled }" />
            </div>
        </div>
        <div class="controls fixed bottom-0 left-0 z-10 border border-white/10 bg-neutral-800 w-full">
            <app-btn text="Add" @click="add" />
            <app-btn text="Remove" :disabled="selectedItemId === null" @click="remove" />
        </div>
    </div>

    <div v-else class="process-list flex flex-col overflow-hidden">
        <div class="row flex justify-between items-center">
            <span class="m-2 text-sm">Select from the list of running apps:</span>
            <app-btn text="Refresh" @click="getProccessList" />
        </div>
        <div class="relative flex flex-col m-2 border border-white/20 h-64 overflow-y-scroll">
            <div class="header flex sticky top-0 bg-[var(--main-bg-color)]">
                <div class="text-sm text-white/40 w-1/2 pl-1 py-1">Process name</div>
                <div class="text-sm text-white/40 w-1/2 pl-1 py-1">Title</div>
            </div>

            <div v-for="(item, i) in processes" :key="i" @click="selectProcess(i)"
                class="item pl-1 py-1 flex b-2 odd:bg-white/5"
                :class="{ 'bg-sky-600 odd:bg-sky-600 text-white': selectedProcessId === i }">
                <span class="w-1/2 text-xs text-sky-500 overflow-hidden text-nowrap pr-2"
                    :class="{ 'text-white': selectedProcessId === i }" v-text="item.filename" />
                <span class="w-1/2 text-xs overflow-hidden text-nowrap" v-text="item.title" />
            </div>
        </div>
        <div class="controls m-2">
            <p class="text-sm">Or type in/browse to the app executable (.exe) file</p>
            <div class="w-full flex items-center">
                <div class="min-w-[80px] text-sm">Process</div>
                <input type="text"
                    class="h-8 flex-grow px-1 text-sm outline-0 border border-transparent text-sky-500 focus:border-sky-500 focus:bg-sky-300/30 focus:text-white"
                    v-model="selectedProcessData.filename">
                <app-btn text="..." @click="browse" />
            </div>
            <div class="w-full flex items-center">
                <div class="min-w-[80px] text-sm">Title</div>
                <input type="text"
                    class="flex w-full h-8 mr-2 px-1 text-sm outline-0 border border-transparent focus:border-sky-500 focus:bg-sky-300/30 focus:text-white"
                    :value="selectedProcessData.title">
            </div>
            <div class="row flex justify-end mt-2">
                <app-btn text="OK" @click="addToBlacklist" />
                <app-btn text="Cancel" @click="isProcessesVisible = false; resetSelected();" />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import AppHeaderbar from "./AppHeaderbar.vue";
import AppBtn from "./AppBtn.vue";
import { nextTick, onBeforeMount, onMounted, onUnmounted, reactive, ref } from "vue";
import { AppItem } from "../common/interfaces";
import { platform } from "@tauri-apps/api/os";
import { open } from '@tauri-apps/api/dialog';
import { getFile, saveTextFile } from "../services/backend";
import { FILE_NAME } from "../common/constants";
import { appWindow } from "@tauri-apps/api/window";

const invoke = window.__TAURI__.invoke;

const selectedItemId = ref<null | number>(null);

const processes = ref<null | AppItem[]>(null);

const blacklistedApps = ref<AppItem[]>([]);

const isProcessesVisible = ref(false);

const selectedProcessId = ref<null | number>(null);

const selectedProcessData = reactive<AppItem>({
    filepath: "",
    enabled: true,
    filename: "",
    title: "",
});

const toggleApp = (item: AppItem) => {
    save();
}

const save = async () => {
    if (await saveTextFile(FILE_NAME.Blacklist, JSON.stringify(blacklistedApps.value))) {
        invoke("update_blacklist_data");
    }
}

const getProccessList = async () => {
    try {
        const response = await invoke('get_proccesses_list');
        const data = JSON.parse(response)

        data.sort((a: AppItem, b: AppItem) => {
            if (!a?.filename || !b?.filename) {
                return;
            }

            if (a?.filename < b?.filename) return -1;
            if (a?.filename > b?.filename) return 1;
            return 0;
        });

        processes.value = data;

        await appWindow.setFocus();
        resetSelected();
    } catch (error) {
        console.error(error);
    }
}

const add = async () => {
    isProcessesVisible.value = true;
    getProccessList();
}

const remove = () => {
    if (selectedItemId.value === null) {
        return;
    }

    blacklistedApps.value?.splice(selectedItemId.value, 1);
    console.log(blacklistedApps.value);

    save();
}

// -----

const resetSelected = () => { selectedProcessId.value = null }

const selectProcess = (i: number) => {
    selectedProcessId.value = i;

    if (processes.value) {
        selectedProcessData.filepath = processes.value[i].filepath;
        selectedProcessData.filename = processes.value[i].filename;
        selectedProcessData.title = processes.value[i].title;
        selectedProcessData.enabled = true;
    }
}

const browse = async () => {
    resetSelected();
    const currentPlatform = await platform();
    console.log(currentPlatform);

    let extensions;
    if (currentPlatform === 'win32') {
        extensions = ['exe'];
    } else if (currentPlatform === 'darwin') {
        extensions = ['app'];
    } else {
        extensions = ['sh'];
    }

    const filepath = await open({
        multiple: false,
        directory: false,
        title: 'Select application',
        filters: [{
            name: 'apps',
            extensions: extensions,
        }]
    });

    if (filepath) {
        console.log('Selected file:', filepath);
        selectedProcessId.value = null;

        selectedProcessData.filepath = filepath as string;
        selectedProcessData.filename = (filepath as string).replace(/^.*(\\|\/|\:)/, '');
        selectedProcessData.enabled = true;
    }
}

const addToBlacklist = () => {
    if (!selectedProcessData.filepath) {
        return;
    }

    blacklistedApps.value.push(selectedProcessData);
    save();

    isProcessesVisible.value = false;
    resetSelected();
}

const loadData = async () => {
    const text = await getFile(FILE_NAME.Blacklist);

    if (text?.length) {
        try {
            blacklistedApps.value = await JSON.parse(text);
        } catch (e) {
            console.error(e);
        }
    } else {
        console.log('ooops');
    }
}

onMounted(() => {
    loadData();
});
</script>
