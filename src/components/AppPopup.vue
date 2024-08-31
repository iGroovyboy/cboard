<template>
  <div
    v-if="type"
    class="menu-wrapper absolute top-0 w-full h-full z-100"
    @click="$emit('close')"
  >
    <ul v-if="type === MENU_TYPE.Context" class="menu">
      <li @click="action(ACTION.DeleteAll)">Delete all</li>
    </ul>

    <ul v-else-if="type === MENU_TYPE.Main" class="menu main">
      <li class="flex">
        <router-link
          :to="{ name: ROUTE.Autoreplace }"
          class="text-white w-full hover:text-white cursor-default"
          >Auto-replacement</router-link
        >
      </li>
      <li class="flex">
        <router-link
          :to="{ name: ROUTE.Blacklist }"
          class="text-white w-full hover:text-white cursor-default"
          >Blacklist apps</router-link
        >
      </li>
      <li class="flex">
        <router-link
          :to="{ name: ROUTE.Settings }"
          class="text-white w-full hover:text-white"
          >Settings</router-link
        >
      </li>
      <li @click="action(ACTION.About)">About</li>
      <li @click="action(ACTION.Quit)">Quit</li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { Folder, MENU_TYPE } from "../common/constants";
import { folderDeleteAll, quit } from "../services/backend";
import { ROUTE } from "../router/routenames";

const props = defineProps({
  type: {
    type: Number,
    default: MENU_TYPE.Context,
  },
  currentFolder: {
    type: Number,
    default: Folder.Clipboard,
  },
});

const emit = defineEmits(["click", "close"]);

const enum ACTION {
  None,
  Quit,
  DeleteAll,
  About,
}

const action = (action: ACTION) => {
  emit("click");

  switch (action) {
    case ACTION.DeleteAll:
      folderDeleteAll(props.currentFolder);
      break;
    case ACTION.About:
      alert(
        "Clipboard manager\n (c) Anton Babintsev, 2023\n https://github.com/iGroovyboy",
      );
      break;
    case ACTION.Quit:
      quit();
      break;
    default:
      break;
  }
};
</script>

<style scoped lang="scss">
.menu {
  @apply absolute border border-neutral-500/30 w-[50%] text-[12px] py-2 backdrop-blur shadow-md bg-white/5;

  &.main {
    @apply top-8 left-auto right-2;
  }

  li {
    @apply mb-2 last:mb-0 px-4 hover:bg-neutral-500/20;
  }
}
</style>
