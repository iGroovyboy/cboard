<script setup lang="ts">
import { ref } from 'vue'

defineProps<{ msg: string }>()

const count = ref(0)
</script>

<template>
  <h1>{{ msg }}</h1>

  <h2>adasda adas</h2>

  <div class="card">
    <button type="button" @click="count++">count is {{ count }}</button>
    <p>
      Edit
      <code>components/HelloWorld.vue</code> to test HMR
    </p>
  </div>

  <p>
    Check out
    <a href="https://vuejs.org/guide/quick-start.html#local" target="_blank"
      >create-vue</a
    >, the official Vue + Vite starter
  </p>
  <p>
    Install
    <a href="https://github.com/johnsoncodehk/volar" target="_blank">Volar</a>
    in your IDE for a better DX
  </p>
  <p class="read-the-docs">Click on the Vite and Vue logos to learn more</p>
</template>

<script lang="ts">
import { register, unregister } from '@tauri-apps/api/globalShortcut';
import { invoke } from '@tauri-apps/api/tauri'


(async() => { 
  console.log('Hello lulz');

// await register('CommandOrControl+v', () => {
//   console.log('Shortcut triggered');
// });

// unregister('Shift+Alt');

// register('Shift+Alt', () => {
//   console.log('LAN switch');
// });

register('Alt+Enter', () => {
  console.log('Alt+Enter pressed!')

  // When using the Tauri API npm package:
  // When using the Tauri global script (if not using the npm package)
  // Be sure to set `build.withGlobalTauri` in `tauri.conf.json` to true
  const invoke = window.__TAURI__.invoke

  // Invoke the command
  invoke('my_custom_command')
})


register('CommandOrControl+c', () => {
  console.log('copy hotkey');
});
 }
)()

</script>

<style scoped>
.read-the-docs {
  color: #888;
}
</style>
