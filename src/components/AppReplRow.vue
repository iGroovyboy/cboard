<template>
  <div class="flex gap-0.5 mb-0.5 w-full">
    <input
      type="text"
      class="bg-white/10 outline-0 w-[47%] text-emerald-500 font-bold px-2 py-1 border border-transparent focus:border-sky-500"
      maxlength="256"
      v-model="key"
      @input="update"
    />
    <input
      type="text"
      class="bg-white/10 outline-0 w-[47%] text-white px-2 py-1 border border-transparent focus:border-sky-500"
      v-model="value"
      @keyup="handleKeyPress"
      @input="update"
    />
    <div class="w-5 flex justify-center items-center">
      <button v-if="isConstructor" @click="add">
        <img
          class="w-5 opacity-50 rotate-45 hover:opacity-100"
          src="../assets/close-outline.svg"
          alt="Add"
        />
      </button>
      <button v-else @click="$emit('remove', key)">
        <img
          class="w-5 opacity-50 hover:opacity-100"
          src="../assets/close-outline.svg"
          alt="Delete"
        />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { AutoReplacementItem } from "../common/interfaces";
import { ref } from "vue";
import { debounce } from "../common/helpers";

interface AppReplRowProps {
  data: AutoReplacementItem;
  isConstructor: boolean;
}

const props = withDefaults(defineProps<AppReplRowProps>(), {
  data: {
    key: "",
    value: "",
  },
  isConstructor: false,
});

const emit = defineEmits(["add", "remove", "update"]);

const key = ref(props.data.key);

const value = ref(props.data.value);

const add = () => {
  emit("add", { key: key.value, value: value.value });

  key.value = "";
  value.value = "";
};

const handleKeyPress = (e: KeyboardEvent) => {
  if (e.key === "Enter") {
    add();
  }
};

const update = debounce(() => {
  emit("update", 
    props.data.key, 
    { key: key.value, value: value.value }
  );
}, 1000);

</script>
