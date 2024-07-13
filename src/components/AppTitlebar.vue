<template>
    <div class="titlebar relative h-8">
        <div data-tauri-drag-region
            class="bar w-full h-full absolute flex justify-center items-center opacity-30 transition hover:opacity-50">
            <div class="drag w-20 h-1 bg-white  rounded-sm pointer-events-none" />
        </div>
        <div class="close absolute right-0 w-10 h-8 flex justify-center items-center opacity-50 transition hover:opacity-100"
            @click="close">
            <img class="w-6" src="../assets/close-outline.svg" alt="close" />
        </div>
    </div>
</template>

<script setup lang="ts">
const close = () => window.__TAURI__.invoke('hide_window');
</script>