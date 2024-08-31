<template>
  <app-headerbar title="Settings" />

  <div class="settings flex flex-col m-1 text-sm gap-y-2">
    <div class="section">
      <h2>Autorun</h2>
      <div class="options flex flex-col gap-y-1">
        <div class="option">
          <input id="autorun_enabled" type="checkbox" v-model="settings.autorun">
          <label for="autorun_enabled">Launch when system starts</label>
        </div>
      </div>
    </div>

    <div class="section" v-if="isWin">
      <h2>Windows key</h2>
      <div class="options flex flex-col gap-y-1">
        <div class="option">
          <input name="win_key" id="win_key_normal" type="radio" :value="0" v-model="settings.win_key">
          <label for="win_key_normal">Normal behaviour</label>
        </div>

        <div class="option">
          <input name="win_key" id="win_key_fullscreen" type="radio" :value="1" v-model="settings.win_key">
          <label for="win_key_fullscreen">Disable in fullscreen mode</label>
        </div>

        <div class="option">
          <input name="win_key" id="win_key_hotkeys" type="radio" :value="2" v-model="settings.win_key">
          <label for="win_key_hotkeys">Replace with hotkeys</label>
          <input class="hotkeys" @click="showHotkey('win_key_hotkey')" id="win_key_hotkeys_data" type="text" readonly
            :value="displayHotkeys(settings.win_key_hotkey)">
        </div>
      </div>
    </div>

    <div class="section">
      <h2>Hotkeys</h2>
      <div class="options flex flex-col gap-y-1">
        <div class="option" @click="showHotkey('show_app_hotkey')">
          <label for="h1">Show app main screen</label>
          <input class="hotkeys" id="h1" type="text" :value="displayHotkeys(settings.show_app_hotkey)">
        </div>
      </div>
    </div>
  </div>

  <div class="controls mx-3 flex justify-end">
    <app-btn text="Save settings" @click="saveSettings" />
    <app-btn text="Cancel" @click="cancelSettings" />
  </div>

  <div v-if="isShowHotkeyReader"
    class="hotkeys fixed w-full h-full bg-neutral-800 flex flex-col justify-center items-center">
    <div class="w-full p-4">
      <input class="w-full p-2 text-center text-green-500 outline-none border select-none" type="text"
        placeholder="Press hotkeys" readonly :value="displayHotkeys(recordedHotkeys)">
    </div>

    <div class="">
      <app-btn text="Save" :disabled="!recordedHotkeys" @click="saveHotkey" />
      <app-btn text="Cancel" @click="cancelHotkey" />
    </div>
  </div>

</template>

<script setup lang="ts">
import { type } from "@tauri-apps/api/os";
import AppHeaderbar from "./AppHeaderbar.vue";
import AppBtn from "./AppBtn.vue";
import { onBeforeMount, onUnmounted, reactive, ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { useRouter } from "vue-router";
import { FILE_NAME } from "../common/constants";
import { getFile, saveTextFile } from "../services/backend";

const invoke = window.__TAURI__.invoke;

const router = useRouter();

const recordedHotkeys = ref(null);

const isShowHotkeyReader = ref(false);

const isWin = ref(true);

const settings = reactive<Record<string, unknown>>({
  autorun: true,

  win_key: 0,
  win_key_hotkey: '',

  show_app_hotkey: 'Ctrl + 1',
});

let currentSettingHotkey: null | string = null;

let hotkeyReaderUnlisten = () => { };

const cancelHotkey = () => {
  isShowHotkeyReader.value = false;
  recordedHotkeys.value = null;
  invoke("hotkeys_unlisten");
  hotkeyReaderUnlisten();
}

const showHotkey = async (p: string) => {
  isShowHotkeyReader.value = true;
  currentSettingHotkey = p;

  const response = await invoke("hotkeys_listen");
  console.log(">", response);

  hotkeyReaderUnlisten = await listen<string>('hotkeys_reader', (event) => {
    console.log('Received data from Rust:', event.payload);
    if (event?.payload?.keys?.length) {
      const keysArr = event.payload.keys.split(',');
      keysArr.sort((a, b) => b.length - a.length);
      recordedHotkeys.value = keysArr;
    }
  });
}

const saveHotkey = (p: string) => {
  settings[currentSettingHotkey] = recordedHotkeys.value.join(',');
  cancelHotkey();
  console.log(settings);
}

const displayHotkeys = (hotkeys: string[] | string) => {
  if (Array.isArray(hotkeys)) {
    return hotkeys?.join(' + ')
  }

  return hotkeys?.replaceAll(',', ' + ');
}

const loadSettings = async () => {
  const text = await getFile(FILE_NAME.Settings);

  if (text?.length) {
    try {
      Object.assign(settings, (await JSON.parse(text)));
      console.log("loaded!", settings);
    } catch (e) {
      console.error(e);
    }
  } else {
    console.log('ooops');
  }
}

const saveSettings = async () => {
  console.log("save>", settings);

  if (await saveTextFile(FILE_NAME.Settings, JSON.stringify(settings))) {
    invoke("update_settings");
    router.back()
  }
}

const cancelSettings = () => {
  loadSettings();
  router.back()
}

onBeforeMount(async () => {
  isWin.value = await type() === "Windows_NT"
  loadSettings();
});

onUnmounted(() => {
  hotkeyReaderUnlisten();
});
</script>

<style scoped lang="scss">
h2 {
  @apply font-bold mb-1;
}

.section {
  @apply mx-4 py-4 border-b border-b-white/20;
}

.option {
  @apply flex justify-start items-center gap-x-2;
}

.hotkeys {
  @apply px-2 py-1 text-amber-500 outline-none
}
</style>
