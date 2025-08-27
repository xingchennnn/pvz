<template>
  <div>{{ windows }}</div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const windows = ref<string[]>([]);

// 请求所有窗口
const getAllWindows = async () => {
  const JsonData = await invoke<string>("get_all_windows");

  let windows = JSON.parse(JsonData);
  // windows.forEach((win: any) => {
  //   win = JSON.parse(win);
  // });
  windows = windows.filter((win: any) => {
    return !["IME", "MSCTFIME UI", "BroadcastListenerWindow"].includes(win.class_name);
  });
  console.log('传递了', windows.length);

  return windows;
};

onMounted(async () => {
  windows.value = await getAllWindows();
  console.log(windows.value);
});
</script>
