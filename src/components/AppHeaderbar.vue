<template>
  <nav
    class="flex flex-row p-3 px-5 justify-between border-b border-gray-500 overflow-hidden"
  >
    <div
      class="title text-xs sm:text-base uppercase font-bold"
      v-text="title"
    />
    <button class="" @click="router.back()">
      <img class="h-5" src="../assets/arrow-back-outline.svg" alt="back" />
    </button>
  </nav>
</template>

<script setup lang="ts">
import { useRouter } from "vue-router";

interface AppHeaderbarProps {
  title: string;
}

const props = withDefaults(defineProps<AppHeaderbarProps>(), {
  title: "Title",
});

const router = useRouter();
</script>

<style lang="scss" scoped></style>
