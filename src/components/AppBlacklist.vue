<template>
  <app-headerbar title="Blacklist applications" />

  <div
    v-if="!isProcessesVisible"
    class="blacklisted-apps flex flex-col overflow-y-scroll flex-grow pb-16"
  >
    <div class="apps pt-0.5 overflow-x-hidden flex flex-col flex-grow">
      <div
        v-for="(item, i) in blacklistedApps"
        :key="i"
        @mousedown="selectedItemId = i"
        class="item flex gap-x-2 px-1 mb-1 items-center leading-5"
        :class="{ 'bg-sky-600 text-white': selectedItemId == i }"
      >
        <input
          class="min-h-4 min-w-4"
          type="checkbox"
          v-model="item.enabled"
          @change="toggleApp(item)"
        />
        <span
          v-text="item.title || item.filename"
          :title="(item.title || item.filename) + `   (${item.filename})`"
          class="text-sm text-nowrap"
          :class="{ 'text-white/40': !item.enabled }"
        />
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
import { AppItem } from "../common/interfaces";
import { getFile, saveTextFile } from "../services/backend";
import { FILE_NAME } from "../common/constants";
import AppProcessList from "./AppProcessList.vue";

// TODO: move code to composable

const invoke = window.__TAURI__.invoke;

const selectedItemId = ref<null | number>(null);

const blacklistedApps = ref<AppItem[]>([]);

const isProcessesVisible = ref(false);

const toggleApp = (item: AppItem) => {
  save();
};

const save = async () => {
  if (
    await saveTextFile(
      FILE_NAME.Blacklist,
      JSON.stringify(blacklistedApps.value),
    )
  ) {
    invoke("update_blacklist_data");
  }
};

const remove = () => {
  if (selectedItemId.value === null) {
    return;
  }

  blacklistedApps.value?.splice(selectedItemId.value, 1);
  console.log(blacklistedApps.value);

  save();
};

const add = (data: AppItem) => {
  isProcessesVisible.value = false;

  blacklistedApps.value.push(data);
  save();
};

const loadData = async () => {
  const text = await getFile(FILE_NAME.Blacklist);
  if (text?.length) {
    try {
      blacklistedApps.value = await JSON.parse(text);
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
