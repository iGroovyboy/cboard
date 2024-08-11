<template>
  <button class="m-2 p-1 px-4 border text-sm" :class="colors" v-text="text" />
</template>

<script setup lang="ts">
import { computed } from 'vue';

interface AppHeaderbarProps {
  text: string;
  color: string;
  disabled: boolean;
}

const props = withDefaults(defineProps<AppHeaderbarProps>(), {
  text: "Title",
  color: "sky-500",
  disabled: false,
});

const colors = computed(() =>
  props.disabled
    ? `text-white/50 border-white/50`
    : `text-${props.color} border-${props.color}`
);
</script>
