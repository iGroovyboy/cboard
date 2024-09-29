<template>
  <app-headerbar title="Keyboard layouts per app" />

  <div
    v-if="!isProcessesVisible"
    class="blacklisted-apps flex flex-col overflow-y-scroll flex-grow pb-16"
  >
    <div class="apps pt-0.5 overflow-x-hidden flex flex-col flex-grow">
      <div
        v-for="(item, i) in apps"
        :key="i"
        @mousedown="selectedItemId = i"
        class="item flex justify-between mb-1"
        :class="{ 'bg-sky-600 text-white': selectedItemId == i }"
      >
        <div
          class="left w-8/12 flex gap-x-2 px-1 mb-1 items-center leading-5 overflow-ellipsis"
        >
          <input
            class="min-h-4 min-w-4"
            type="checkbox"
            v-model="item.enabled"
            @change="toggleApp(item)"
          />
          <span
            v-text="item.title || item.filename"
            :title="(item.title || item.filename) + ` (${item.filename})`"
            class="text-sm text-nowrap"
            :class="{ 'text-white/40': !item.enabled }"
          />
        </div>
        <div class="right w-4/12">
          <select
            v-if="availableLayouts.length"
            class="text-sm p-1 outline-none"
            @change="changeLang($event, i)"
          >
            <option
              v-for="(layout, id) in availableLayouts"
              :key="layout.lang_id"
              :value="layout.lang_id"
              v-text="layout.lang_name + ` (${layout.lang_code})`"
              :selected="item.lang_id === layout.lang_id"
            />
          </select>
        </div>
      </div>
    </div>
    <div
      class="controls fixed bottom-0 left-0 z-10 border border-white/10 bg-neutral-800 w-full"
    >
      <app-btn text="Add" @click="isProcessesVisible = true" />
      <app-btn
        text="Remove"
        :disabled="selectedItemId === null"
        @click="remove"
      />
    </div>
  </div>

  <AppProcessList v-else @add="add" @cancel="isProcessesVisible = false" />
</template>

<script setup lang="ts">
import AppHeaderbar from "./AppHeaderbar.vue";
import AppBtn from "./AppBtn.vue";
import { onMounted, ref } from "vue";
import { AppItem, KeyAppItem, KeyboardLayout } from "../common/interfaces";
import { getFile, saveTextFile } from "../services/backend";
import { FILE_NAME } from "../common/constants";
import AppProcessList from "./AppProcessList.vue";

const invoke = window.__TAURI__.invoke;

const selectedItemId = ref<null | number>(null);

const availableLayouts = ref<KeyboardLayout[]>([]);

const apps = ref<KeyAppItem[]>([]);

const isProcessesVisible = ref(false);

const toggleApp = () => {
  save();
};

const changeLang = (event, i: number) => {
  apps.value[i].lang_id = +event.target.value;
  save();
};

const fillEmptyLanguages = (arr: KeyAppItem[]) => {
  for (const app of arr) {
    if (!app.lang_id) {
      app.lang_id = availableLayouts.value[0].lang_id || 0;
    }
  }
};

const save = async () => {
  fillEmptyLanguages(apps.value);

  if (
    await saveTextFile(FILE_NAME.KeyboardLayouts, JSON.stringify(apps.value))
  ) {
    invoke("update_keyboard_layouts_data");
  }
};

const add = (data: AppItem) => {
  isProcessesVisible.value = false;
  (data as KeyAppItem).lang_id = availableLayouts.value[0].lang_id;
  apps.value.push(data);
  save();
};

const remove = () => {
  if (selectedItemId.value === null) {
    return;
  }

  apps.value?.splice(selectedItemId.value, 1);
  console.log(apps.value);

  save();
};

const loadData = async () => {
  availableLayouts.value = await invoke("get_available_keyboard_layouts");

  const text = await getFile(FILE_NAME.KeyboardLayouts);

  if (text?.length) {
    try {
      apps.value = await JSON.parse(text);
      console.log("data", apps.value);
    } catch (e) {
      console.error(e);
    }
  } else {
    console.log("ooops");
  }
};

onMounted(() => {
  loadData();
});
</script>
