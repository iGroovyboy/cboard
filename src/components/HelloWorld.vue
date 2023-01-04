<template>
  <h1>{{ msg }}</h1>

  <h2>adasda adas</h2>11

</template>

<script setup lang="ts">
import { ref } from 'vue'

defineProps<{ msg: string }>()

const count = ref(0)
</script>


<script lang="ts">

</script>

<style scoped>
.read-the-docs {
  color: #888;
}
</style>
