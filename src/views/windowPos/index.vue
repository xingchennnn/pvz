<template>
  <div>
    <h2 class="text-lg font-bold mb-2">窗口列表</h2>
    <ul class="list-none p-0">
      <li
        v-for="(win, index) in windows"
        :key="index"
        :class="`border-1 
         bg-[var(--text-color)] 
        c-[var(--bg-color)] p-1 mb-2 rounded 
        flex flex-col cursor-pointer 
        hover:bg-[#f0f0f0] 
        ${win.isActive ? 'border-[#007bff]' : ''}
        `"
        @click="setActiveWindow(win)"
      >
        <span>标题: {{ win.title }}</span>
        <span>类名: {{ win.class_name }}</span>
        <span>句柄: {{ win.hwnd }}</span>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const windows = ref<
  { title: string; class_name: string; hwnd: number; isActive: boolean }[]
>([]);

// 请求所有窗口
const getAllWindows = async () => {
  const JsonData = await invoke<string>("get_all_windows");
  let windows = JSON.parse(JsonData);

  windows = windows.filter((win: any) => {
    return !["IME", "MSCTFIME UI", "BroadcastListenerWindow"].includes(
      win.class_name
    );
  });

  return windows;
};

const setActiveWindow = (win: any) => {
  win.isActive = !win.isActive;
  // invoke("set_window_pos", {
  //   hwnd: win.hwnd,
  //   insertAfter: win.isActive ? -1 : -2, // -1 置顶 -2 取消置顶
  //   x: 0,
  //   y: 0,
  //   cx: 0,
  //   cy: 0,
  //   uFlags: 3, // SWP_NOSIZE | SWP_NOMOVE
  // });
};

onMounted(async () => {
  windows.value = await getAllWindows();
  console.log(windows.value);
});
</script>
