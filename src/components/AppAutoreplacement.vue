<template>
  <app-headerbar title="Auto-replacement" />
  <main
    class="relative w-full flex flex-col p-1 overflow-x-hidden overflow-y-scroll h-[calc(100vh-5rem)]"
  >
    <app-repl-row
      v-for="(row, i) in data"
      :key="i"
      :data="row"
      @remove="remove"
    />

    <app-repl-row class="mt-1" isConstructor @add="add" />
  </main>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { AutoReplacementItem } from "../common/interfaces";
import AppHeaderbar from "./AppHeaderbar.vue";
import AppReplRow from "./AppReplRow.vue";
import { getFile, saveTextFile } from "../services/backend";
import { FILE_NAME } from "../common/constants";

const invoke = window.__TAURI__.invoke;

const data = ref<AutoReplacementItem[]>([]);

const save = async () => {
  if (await saveTextFile(FILE_NAME.Autoreplace, JSON.stringify(data.value))) {
    invoke("update_auto_replace_data");
  }
};

const add = async (item: AutoReplacementItem) => {
  if (!item.key.length) {
    return;
  }

  if (data.value.find((i) => i.key === item.key)) {
    return;
  }

  data.value.push(item);
  await save();
};

const remove = async (key: string) => {
  data.value = data.value.filter((r: AutoReplacementItem) => r.key !== key);
  await save();
};

// TODO: mb add event listeners for keyboard controls
onMounted(async () => {
  const text = await getFile(FILE_NAME.Autoreplace);
  if (text?.length) {
    try {
      data.value = JSON.parse(text);
      console.log("data", data.value);
    } catch (e) {
      console.error(e);
    }
  } else {
    data.value.push({
      key: "<3",
      value: "❤️",
    });
  }
});
</script>

<style scoped lang="scss"></style>
