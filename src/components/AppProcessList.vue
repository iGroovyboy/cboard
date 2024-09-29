<template>
  <div class="process-list flex flex-col overflow-hidden">
    <div class="row flex justify-between items-center">
      <span class="m-2 text-sm">Select from the list of running apps:</span>
      <app-btn text="Refresh" @click="getProccessList" />
    </div>
    <div
      class="relative flex flex-col m-2 border border-white/20 h-64 overflow-y-scroll"
    >
      <div class="header flex sticky top-0 bg-[var(--main-bg-color)]">
        <div class="text-sm text-white/40 w-1/2 pl-1 py-1">Process name</div>
        <div class="text-sm text-white/40 w-1/2 pl-1 py-1">Title</div>
      </div>

      <div
        v-for="(item, i) in processes"
        :key="i"
        @click="selectProcess(i)"
        class="item pl-1 py-1 flex b-2 odd:bg-white/5"
        :class="{
          'bg-sky-600 odd:bg-sky-600 text-white': selectedProcessId === i,
        }"
      >
        <span
          class="w-1/2 text-xs text-sky-500 overflow-hidden text-nowrap pr-2"
          :class="{ 'text-white': selectedProcessId === i }"
          v-text="item.filename"
        />
        <span
          class="w-1/2 text-xs overflow-hidden text-nowrap"
          v-text="item.title"
        />
      </div>
    </div>
    <div class="controls m-2">
      <p class="text-sm">Or type in/browse to the app executable (.exe) file</p>
      <div class="w-full flex items-center">
        <div class="min-w-[80px] text-sm">Process</div>
        <input
          type="text"
          class="h-8 flex-grow px-1 text-sm outline-0 border border-transparent text-sky-500 focus:border-sky-500 focus:bg-sky-300/30 focus:text-white"
          v-model="selectedProcessData.filename"
        />
        <app-btn text="..." @click="browse" />
      </div>
      <div class="w-full flex items-center">
        <div class="min-w-[80px] text-sm">Title</div>
        <input
          type="text"
          class="flex w-full h-8 mr-2 px-1 text-sm outline-0 border border-transparent focus:border-sky-500 focus:bg-sky-300/30 focus:text-white"
          :value="selectedProcessData.title"
        />
      </div>
      <div class="row flex justify-end mt-2">
        <app-btn text="OK" @click="addToList" />
        <app-btn text="Cancel" @click="cancel" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import AppBtn from "./AppBtn.vue";
import { AppItem } from "../common/interfaces";
import { appWindow } from "@tauri-apps/api/window";
import { onMounted, reactive, ref } from "vue";
import { platform } from "@tauri-apps/api/os";
import { open } from "@tauri-apps/api/dialog";

const emit = defineEmits<{
  (e: "add", id: number): void;
  (e: "cancel"): void;
}>();

const invoke = window.__TAURI__.invoke;

const processes = ref<null | AppItem[]>(null);

const selectedProcessId = ref<null | number>(null);

const selectedProcessData = reactive<AppItem>({
  filepath: "",
  enabled: true,
  filename: "",
  title: "",
});

const getProccessList = async () => {
  try {
    const response = await invoke("get_proccesses_list");
    const data = JSON.parse(response);

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
};

const selectProcess = (i: number) => {
  selectedProcessId.value = i;

  if (processes.value) {
    selectedProcessData.filepath = processes.value[i].filepath;
    selectedProcessData.filename = processes.value[i].filename;
    selectedProcessData.title = processes.value[i].title;
    selectedProcessData.enabled = true;
  }
};

const addToList = () => {
  if (!selectedProcessData.filepath) {
    return;
  }

  emit("add", selectedProcessData);
};

const cancel = () => {
  emit("cancel");
};

const browse = async () => {
  resetSelected();
  const currentPlatform = await platform();
  console.log(currentPlatform);

  let extensions;
  if (currentPlatform === "win32") {
    extensions = ["exe"];
  } else if (currentPlatform === "darwin") {
    extensions = ["app"];
  } else {
    extensions = ["sh"];
  }

  const filepath = await open({
    multiple: false,
    directory: false,
    title: "Select application",
    filters: [
      {
        name: "apps",
        extensions: extensions,
      },
    ],
  });

  if (filepath) {
    console.log("Selected file:", filepath);
    selectedProcessId.value = null;

    selectedProcessData.filepath = filepath as string;
    selectedProcessData.filename = (filepath as string).replace(
      /^.*(\\|\/|\:)/,
      "",
    );
    selectedProcessData.enabled = true;
  }
};

const resetSelected = () => {
  selectedProcessId.value = null;
};

onMounted(() => {
  resetSelected();
  getProccessList();
});
</script>
